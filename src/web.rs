// izone/src/web.rs
//
// `izone web`: a small local web server with a JSON API and a single-page UI.
// All device access goes through one mutex so requests reach the controller one at a time.

use std::collections::HashMap;
use std::io::Read;
use std::net::IpAddr;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::OsRng;

use colored::Colorize;
use reqwest::blocking::Client;
use serde_json::{json, Map, Value};
use tiny_http::{Header, Method, Request, Response, Server};

use crate::api::{build_client, make_query_request, try_command_request};
use crate::constants::IZONE_IP;
use crate::web_auth;

const UI_HTML: &str = include_str!("web_ui.html");
const MAX_BODY: u64 = 64 * 1024;
const MAX_SCHEDULES: u8 = 9;
const MAX_ZONES: u8 = 14;
const SLEEP_TIMER_MINUTES: [u64; 8] = [0, 30, 60, 120, 240, 360, 480, 720];
/// Fields never sent to the browser (controller config password and lock code).
const HIDDEN_SYSTEM_FIELDS: [&str; 2] = ["Pass", "LockCode"];

struct Ctx {
    client: Client,
    device: Mutex<()>,
    auth: Mutex<Auth>,
    warnings: Mutex<WarningSamples>,
    last_activity: Mutex<Instant>,
    port: u16,
    loopback: bool,
}

// ----------------------------------------------------------------------------
// Warnings: some controllers flicker `Warnings` (e.g. "filter" <-> "none" every few seconds)
// while the wall controller shows nothing. Only report a warning that holds steady.
// ----------------------------------------------------------------------------

const WARN_WINDOW: Duration = Duration::from_secs(30);
const WARN_MIN_SAMPLES: usize = 5;
const WARN_SAMPLE_EVERY: Duration = Duration::from_secs(5);
const WARN_ACTIVE_FOR: Duration = Duration::from_secs(120); // sample only while the UI is in use

#[derive(Default)]
struct WarningSamples {
    samples: Vec<(Instant, String)>,
}

impl WarningSamples {
    fn record(&mut self, warning: &str) {
        self.record_at(Instant::now(), warning)
    }

    fn record_at(&mut self, now: Instant, warning: &str) {
        self.samples.retain(|(t, _)| now.duration_since(*t) <= WARN_WINDOW * 2);
        self.samples.push((now, warning.to_string()));
    }

    /// The warning if every sample over the last WARN_WINDOW agrees on it, else "none".
    fn steady(&self) -> String {
        self.steady_at(Instant::now())
    }

    fn steady_at(&self, now: Instant) -> String {
        let recent: Vec<&(Instant, String)> = self.samples.iter().filter(|(t, _)| now.duration_since(*t) <= WARN_WINDOW).collect();
        let covers_window = recent.first().map_or(false, |(t, _)| now.duration_since(*t) >= WARN_WINDOW - WARN_SAMPLE_EVERY);
        match recent.first() {
            Some((_, w)) if w != "none" && covers_window && recent.len() >= WARN_MIN_SAMPLES && recent.iter().all(|(_, x)| x == w) => w.clone(),
            _ => "none".to_string(),
        }
    }
}

// ----------------------------------------------------------------------------
// Authentication state
// ----------------------------------------------------------------------------

const MAX_FAILURES: u32 = 5; // per client address
const GLOBAL_MAX_FAILURES: usize = 20; // across all addresses, within LOCKOUT
const LOCKOUT: Duration = Duration::from_secs(15 * 60);
const SESSION_TTL: Duration = Duration::from_secs(7 * 24 * 3600);
const MAX_SESSIONS: usize = 100;
const COOKIE: &str = "izone_session";

struct Session {
    expires: Instant,
    must_change: bool,
}

#[derive(Default)]
struct Failures {
    count: u32,
    locked_until: Option<Instant>,
}

/// The password hash in effect: the stored (changed) one, or the compiled-in default.
enum Credential {
    Default,
    Custom(String),
    Broken(String), // stored file exists but can't be read
}

struct Auth {
    credential: Credential,
    stamp: Option<SystemTime>,
    sessions: HashMap<String, Session>,
    failures: HashMap<IpAddr, Failures>,
    recent_failures: Vec<Instant>,
}

impl Auth {
    fn load() -> Auth {
        let mut a = Auth {
            credential: Credential::Default,
            stamp: None,
            sessions: HashMap::new(),
            failures: HashMap::new(),
            recent_failures: Vec::new(),
        };
        a.reload();
        a
    }

    fn reload(&mut self) {
        self.stamp = web_auth::credential_stamp();
        self.credential = match web_auth::load_password_hash() {
            Ok(Some(h)) => Credential::Custom(h),
            Ok(None) => Credential::Default,
            Err(e) => Credential::Broken(e),
        };
    }

    /// Pick up changes made outside this process (e.g. `izone web reset-password`);
    /// any change signs everyone out.
    fn refresh(&mut self) {
        if web_auth::credential_stamp() != self.stamp {
            self.reload();
            self.sessions.clear();
        }
    }

    fn hash(&self) -> Result<&str, String> {
        match &self.credential {
            Credential::Default => Ok(web_auth::DEFAULT_PASSWORD_HASH),
            Credential::Custom(h) => Ok(h),
            Credential::Broken(e) => Err(e.clone()),
        }
    }

    fn is_default(&self) -> bool {
        matches!(self.credential, Credential::Default)
    }

    /// Seconds until this address (or everyone) may try again, if locked out.
    fn locked_for(&mut self, ip: IpAddr) -> Option<u64> {
        let now = Instant::now();
        self.recent_failures.retain(|t| now.duration_since(*t) < LOCKOUT);
        let mut wait = None;
        if self.recent_failures.len() >= GLOBAL_MAX_FAILURES {
            let oldest = self.recent_failures[0];
            wait = Some((oldest + LOCKOUT).saturating_duration_since(now).as_secs() + 1);
        }
        if let Some(f) = self.failures.get_mut(&ip) {
            match f.locked_until {
                Some(until) if until > now => {
                    let w = until.duration_since(now).as_secs() + 1;
                    wait = Some(wait.map_or(w, |x: u64| x.max(w)));
                }
                Some(_) => *f = Failures::default(), // lock expired: start over
                None => {}
            }
        }
        wait
    }

    /// Record a failed attempt; returns attempts left before lockout (0 = now locked).
    fn record_failure(&mut self, ip: IpAddr) -> u32 {
        let now = Instant::now();
        self.recent_failures.push(now);
        if self.failures.len() > 10_000 {
            self.failures.retain(|_, f| f.locked_until.map_or(false, |u| u > now));
        }
        let f = self.failures.entry(ip).or_default();
        f.count += 1;
        if f.count >= MAX_FAILURES {
            f.locked_until = Some(now + LOCKOUT);
            0
        } else {
            MAX_FAILURES - f.count
        }
    }

    fn new_session(&mut self, must_change: bool) -> String {
        let now = Instant::now();
        self.sessions.retain(|_, s| s.expires > now);
        if self.sessions.len() >= MAX_SESSIONS {
            // Drop the session closest to expiry.
            if let Some(k) = self.sessions.iter().min_by_key(|(_, s)| s.expires).map(|(k, _)| k.clone()) {
                self.sessions.remove(&k);
            }
        }
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        let token: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        self.sessions.insert(token.clone(), Session { expires: now + SESSION_TTL, must_change });
        token
    }

    fn session(&mut self, token: Option<&str>) -> Option<&mut Session> {
        let token = token?;
        let now = Instant::now();
        let valid = self.sessions.get(token).map_or(false, |s| s.expires > now);
        if !valid {
            self.sessions.remove(token);
            return None;
        }
        self.sessions.get_mut(token)
    }
}

type ApiResult = Result<Value, (u16, String)>;

fn bad(msg: impl Into<String>) -> (u16, String) {
    (400, msg.into())
}
fn device_err(msg: impl Into<String>) -> (u16, String) {
    (502, msg.into())
}

pub fn run(host: &str, port: u16) {
    if !web_auth::default_hash_is_valid() {
        eprintln!("{}", "Error: no default web password is configured in this build.".red());
        eprintln!("Run `izone web hash-password`, then put the printed hash in DEFAULT_PASSWORD_HASH (src/web_auth.rs) and rebuild.");
        exit(1);
    }
    let auth = Auth::load();
    let addr = format!("{}:{}", host, port);
    let server = match Server::http(&addr) {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("{}", format!("Error: could not start the web server on {}: {}", addr, e).red());
            eprintln!("Try another port, e.g. izone web --port 8081");
            exit(1);
        }
    };
    let loopback = matches!(host, "127.0.0.1" | "localhost" | "::1" | "[::1]");
    let shown_host = if host == "127.0.0.1" { "localhost" } else { host };
    println!("iZone web UI running at {}", format!("http://{}:{}/", shown_host, port).green().bold());
    println!("Controller: {}", &**IZONE_IP);
    match &auth.credential {
        Credential::Default => println!("Password: {} (you'll be asked to change it after signing in)", "default".yellow()),
        Credential::Custom(_) => println!("Password: custom, stored in {}", web_auth::storage_description()),
        Credential::Broken(e) => println!("{}", format!("Warning: {}", e).red()),
    }
    if !loopback {
        println!(
            "{}",
            "Note: listening on a non-local address. Traffic is plain HTTP: use a VPN, or an HTTPS reverse proxy before exposing it to the internet."
                .yellow()
        );
    }
    println!("Press Ctrl+C to stop.");

    let ctx = Arc::new(Ctx {
        client: build_client(),
        device: Mutex::new(()),
        auth: Mutex::new(auth),
        warnings: Mutex::new(WarningSamples::default()),
        last_activity: Mutex::new(Instant::now() - WARN_ACTIVE_FOR),
        port,
        loopback,
    });
    // Background warning sampler: only while someone is using the UI.
    {
        let ctx = Arc::clone(&ctx);
        thread::spawn(move || loop {
            thread::sleep(WARN_SAMPLE_EVERY);
            let active = ctx.last_activity.lock().map(|t| t.elapsed() < WARN_ACTIVE_FOR).unwrap_or(false);
            if active {
                let _guard = ctx.device.lock().unwrap_or_else(|p| p.into_inner());
                let _ = system(&ctx);
            }
        });
    }
    let mut workers = Vec::new();
    for _ in 0..4 {
        let server = Arc::clone(&server);
        let ctx = Arc::clone(&ctx);
        workers.push(thread::spawn(move || loop {
            match server.recv() {
                Ok(req) => handle(&ctx, req),
                Err(e) => eprintln!("web: failed to receive request: {}", e),
            }
        }));
    }
    for w in workers {
        let _ = w.join();
    }
}

fn header(req: &Request, name: &str) -> Option<String> {
    req.headers()
        .iter()
        .find(|h| h.field.as_str().as_str().eq_ignore_ascii_case(name))
        .map(|h| h.value.as_str().to_string())
}

fn hdr(name: &str, value: &str) -> Header {
    Header::from_bytes(name.as_bytes(), value.as_bytes()).expect("valid header")
}

fn respond_json(req: Request, status: u16, body: Value) {
    respond_json_with(req, status, body, Vec::new())
}

fn respond_json_with(req: Request, status: u16, body: Value, extra: Vec<Header>) {
    let mut resp = Response::from_string(body.to_string())
        .with_status_code(status)
        .with_header(hdr("Content-Type", "application/json; charset=utf-8"))
        .with_header(hdr("Cache-Control", "no-store"))
        .with_header(hdr("X-Content-Type-Options", "nosniff"));
    for h in extra {
        resp.add_header(h);
    }
    let _ = req.respond(resp);
}

fn respond_html(req: Request) {
    let resp = Response::from_string(UI_HTML)
        .with_header(hdr("Content-Type", "text/html; charset=utf-8"))
        .with_header(hdr("Cache-Control", "no-store"))
        .with_header(hdr("X-Content-Type-Options", "nosniff"))
        .with_header(hdr("X-Frame-Options", "DENY"))
        .with_header(hdr("Referrer-Policy", "no-referrer"))
        .with_header(hdr(
            "Content-Security-Policy",
            "default-src 'none'; script-src 'unsafe-inline'; style-src 'unsafe-inline'; connect-src 'self'; img-src data:; base-uri 'none'; form-action 'none'; frame-ancestors 'none'",
        ));
    let _ = req.respond(resp);
}

/// Reject requests whose Host header isn't ours (protects against DNS rebinding).
fn host_allowed(ctx: &Ctx, req: &Request) -> bool {
    if !ctx.loopback {
        return true;
    }
    let Some(host) = header(req, "Host") else { return false };
    let host = host.to_lowercase();
    let allowed = ["localhost", "127.0.0.1", "[::1]"];
    allowed.iter().any(|h| host == *h || host == format!("{}:{}", h, ctx.port))
}

fn handle(ctx: &Ctx, mut req: Request) {
    if !host_allowed(ctx, &req) {
        return respond_json(req, 403, json!({"ok": false, "error": "Forbidden host"}));
    }
    let method = req.method().clone();
    let path = req.url().split('?').next().unwrap_or("/").to_string();

    if method == Method::Get && (path == "/" || path == "/index.html") {
        return respond_html(req);
    }
    if !path.starts_with("/api/") {
        return respond_json(req, 404, json!({"ok": false, "error": "Not found"}));
    }

    let mut body = Value::Null;
    if method == Method::Post {
        // A custom header can't be sent cross-site without a CORS preflight (which we never allow),
        // so this blocks other websites from sending commands through the user's browser.
        if header(&req, "X-Izone-Web").as_deref() != Some("1") {
            return respond_json(req, 403, json!({"ok": false, "error": "Missing X-Izone-Web header"}));
        }
        let mut text = String::new();
        if req.as_reader().take(MAX_BODY).read_to_string(&mut text).is_err() {
            return respond_json(req, 400, json!({"ok": false, "error": "Could not read request body"}));
        }
        if !text.trim().is_empty() {
            body = match serde_json::from_str(&text) {
                Ok(v) => v,
                Err(e) => return respond_json(req, 400, json!({"ok": false, "error": format!("Invalid JSON: {}", e)})),
            };
        }
    }

    // ---- Authentication ----
    let ip = client_ip(&req);
    let token = cookie(&req, COOKIE);
    let secure = header(&req, "X-Forwarded-Proto").map_or(false, |p| p.eq_ignore_ascii_case("https"));
    match (&method, path.as_str()) {
        (Method::Get, "/api/auth/status") => return auth_status(ctx, req, token.as_deref()),
        (Method::Post, "/api/auth/login") => return auth_login(ctx, req, ip, &body, secure),
        (Method::Post, "/api/auth/logout") => return auth_logout(ctx, req, token.as_deref(), secure),
        (Method::Post, "/api/auth/password") => return auth_change(ctx, req, ip, token.as_deref(), &body),
        _ => {}
    }
    {
        let mut auth = ctx.auth.lock().unwrap_or_else(|p| p.into_inner());
        auth.refresh();
        match auth.session(token.as_deref()) {
            None => return respond_json(req, 401, json!({"ok": false, "error": "Please sign in", "auth": "login"})),
            Some(s) if s.must_change => {
                return respond_json(req, 403, json!({"ok": false, "error": "Change the default password to continue", "auth": "change"}))
            }
            Some(_) => {}
        }
    }
    *ctx.last_activity.lock().unwrap_or_else(|p| p.into_inner()) = Instant::now();

    let segments: Vec<&str> = path.trim_start_matches("/api/").trim_end_matches('/').split('/').collect();
    let result: ApiResult = {
        // One request at a time to the controller.
        let _guard = ctx.device.lock().unwrap_or_else(|p| p.into_inner());
        match (&method, segments.as_slice()) {
            (Method::Get, ["system"]) => get_system(ctx),
            (Method::Get, ["zones"]) => get_zones(ctx),
            (Method::Get, ["schedules"]) => get_schedules(ctx),
            (Method::Get, ["settings"]) => get_settings(ctx),
            (Method::Post, ["system"]) => post_system(ctx, &body),
            (Method::Post, ["zones", i]) => parse_index(i, MAX_ZONES).and_then(|i| post_zone(ctx, i, &body)),
            (Method::Post, ["schedules", i]) => parse_index(i, MAX_SCHEDULES).and_then(|i| post_schedule(ctx, i, &body)),
            (Method::Post, ["schedules", i, "clear"]) => parse_index(i, MAX_SCHEDULES).and_then(|i| clear_schedule(ctx, i)),
            (Method::Post, ["settings"]) => post_setting(ctx, &body),
            _ => Err((404, "Not found".to_string())),
        }
    };

    if method == Method::Post {
        match &result {
            Ok(_) => println!("{} {} {}", "✓".green(), path, body),
            Err((_, e)) => println!("{} {} {} -> {}", "✗".red(), path, body, e.lines().next().unwrap_or("")),
        }
    }
    match result {
        Ok(data) => respond_json(req, 200, json!({"ok": true, "data": data})),
        Err((status, error)) => respond_json(req, status, json!({"ok": false, "error": error})),
    }
}

// ----------------------------------------------------------------------------
// Authentication endpoints
// ----------------------------------------------------------------------------

/// Client address. X-Forwarded-For is only trusted from a reverse proxy on this machine.
fn client_ip(req: &Request) -> IpAddr {
    let peer = req.remote_addr().map(|a| a.ip()).unwrap_or(IpAddr::from([0, 0, 0, 0]));
    if peer.is_loopback() {
        if let Some(fwd) = header(req, "X-Forwarded-For") {
            if let Some(ip) = fwd.split(',').next().and_then(|s| s.trim().parse().ok()) {
                return ip;
            }
        }
    }
    peer
}

fn cookie(req: &Request, name: &str) -> Option<String> {
    header(req, "Cookie")?.split(';').find_map(|kv| {
        let (k, v) = kv.trim().split_once('=')?;
        (k == name && !v.is_empty()).then(|| v.to_string())
    })
}

fn session_cookie(token: &str, max_age: u64, secure: bool) -> Header {
    hdr(
        "Set-Cookie",
        &format!(
            "{}={}; Path=/; HttpOnly; SameSite=Strict; Max-Age={}{}",
            COOKIE,
            token,
            max_age,
            if secure { "; Secure" } else { "" }
        ),
    )
}

fn password_field<'a>(b: &'a Value, key: &str) -> Option<&'a str> {
    b.get(key).and_then(Value::as_str).filter(|s| s.len() <= 1024)
}

fn lockout_message(secs: u64) -> String {
    let mins = (secs + 59) / 60;
    format!("Too many failed attempts. Try again in {} minute{}.", mins, if mins == 1 { "" } else { "s" })
}

fn auth_status(ctx: &Ctx, req: Request, token: Option<&str>) {
    let mut auth = ctx.auth.lock().unwrap_or_else(|p| p.into_inner());
    auth.refresh();
    let (authenticated, must_change) = match auth.session(token) {
        Some(s) => (true, s.must_change),
        None => (false, false),
    };
    drop(auth);
    respond_json(req, 200, json!({"ok": true, "data": {"authenticated": authenticated, "must_change": must_change}}))
}

fn auth_login(ctx: &Ctx, req: Request, ip: IpAddr, body: &Value, secure: bool) {
    let Some(password) = password_field(body, "password") else {
        return respond_json(req, 400, json!({"ok": false, "error": "Password is required"}));
    };
    let mut auth = ctx.auth.lock().unwrap_or_else(|p| p.into_inner());
    auth.refresh();
    if let Some(wait) = auth.locked_for(ip) {
        drop(auth);
        println!("{} sign-in from {} refused (locked out)", "✗".red(), ip);
        return respond_json(req, 429, json!({"ok": false, "error": lockout_message(wait), "retry_after": wait}));
    }
    let hash = match auth.hash() {
        Ok(h) => h.to_string(),
        Err(e) => {
            drop(auth);
            return respond_json(req, 500, json!({"ok": false, "error": e}));
        }
    };
    if web_auth::verify_password(password, &hash) {
        auth.failures.remove(&ip);
        let must_change = auth.is_default();
        let token = auth.new_session(must_change);
        drop(auth);
        println!("{} signed in from {}", "✓".green(), ip);
        let cookie = session_cookie(&token, SESSION_TTL.as_secs(), secure);
        respond_json_with(req, 200, json!({"ok": true, "data": {"must_change": must_change}}), vec![cookie])
    } else {
        let left = auth.record_failure(ip);
        drop(auth);
        println!("{} failed sign-in from {} ({} attempt(s) left)", "✗".red(), ip, left);
        if left == 0 {
            respond_json(req, 429, json!({"ok": false, "error": lockout_message(LOCKOUT.as_secs()), "retry_after": LOCKOUT.as_secs()}))
        } else {
            respond_json(
                req,
                401,
                json!({"ok": false, "error": format!("Wrong password. {} attempt{} left before a 15-minute lockout.", left, if left == 1 { "" } else { "s" })}),
            )
        }
    }
}

fn auth_logout(ctx: &Ctx, req: Request, token: Option<&str>, secure: bool) {
    if let Some(t) = token {
        ctx.auth.lock().unwrap_or_else(|p| p.into_inner()).sessions.remove(t);
    }
    respond_json_with(req, 200, json!({"ok": true, "data": null}), vec![session_cookie("", 0, secure)])
}

fn auth_change(ctx: &Ctx, req: Request, ip: IpAddr, token: Option<&str>, body: &Value) {
    let (Some(current), Some(new)) = (password_field(body, "current"), password_field(body, "new")) else {
        return respond_json(req, 400, json!({"ok": false, "error": "Current and new password are required"}));
    };
    let mut auth = ctx.auth.lock().unwrap_or_else(|p| p.into_inner());
    auth.refresh();
    let Some(token) = token.filter(|t| auth.sessions.contains_key(*t)).map(str::to_string) else {
        drop(auth);
        return respond_json(req, 401, json!({"ok": false, "error": "Please sign in", "auth": "login"}));
    };
    if auth.session(Some(&token)).is_none() {
        drop(auth);
        return respond_json(req, 401, json!({"ok": false, "error": "Please sign in", "auth": "login"}));
    }
    if let Some(wait) = auth.locked_for(ip) {
        drop(auth);
        return respond_json(req, 429, json!({"ok": false, "error": lockout_message(wait), "retry_after": wait}));
    }
    let hash = match auth.hash() {
        Ok(h) => h.to_string(),
        Err(e) => {
            drop(auth);
            return respond_json(req, 500, json!({"ok": false, "error": e}));
        }
    };
    if !web_auth::verify_password(current, &hash) {
        let left = auth.record_failure(ip);
        drop(auth);
        println!("{} wrong current password from {} ({} attempt(s) left)", "✗".red(), ip, left);
        let msg = if left == 0 { lockout_message(LOCKOUT.as_secs()) } else { format!("Current password is wrong. {} attempt{} left.", left, if left == 1 { "" } else { "s" }) };
        return respond_json(req, if left == 0 { 429 } else { 401 }, json!({"ok": false, "error": msg}));
    }
    if new == current {
        drop(auth);
        return respond_json(req, 400, json!({"ok": false, "error": "The new password must be different"}));
    }
    if let Err(e) = web_auth::check_new_password(new) {
        drop(auth);
        return respond_json(req, 400, json!({"ok": false, "error": e}));
    }
    let saved = web_auth::hash_password(new).and_then(|h| web_auth::save_password_hash(&h));
    if let Err(e) = saved {
        drop(auth);
        eprintln!("{} could not save the new web password: {}", "✗".red(), e);
        return respond_json(req, 500, json!({"ok": false, "error": format!("Could not save the new password: {}", e)}));
    }
    auth.reload();
    auth.failures.remove(&ip);
    // Sign out every other session; this one continues.
    auth.sessions.retain(|k, _| *k == token);
    if let Some(s) = auth.sessions.get_mut(&token) {
        s.must_change = false;
    }
    drop(auth);
    println!("{} web password changed from {} (other sessions signed out)", "✓".green(), ip);
    respond_json(req, 200, json!({"ok": true, "data": {"must_change": false}}))
}

fn parse_index(s: &str, max: u8) -> Result<u8, (u16, String)> {
    match s.parse::<u8>() {
        Ok(i) if i < max => Ok(i),
        _ => Err(bad(format!("Index must be 0-{}", max - 1))),
    }
}

// ----------------------------------------------------------------------------
// Device access
// ----------------------------------------------------------------------------

fn query(ctx: &Ctx, kind: u8, no: u8, key: &str) -> Result<Value, (u16, String)> {
    let v = make_query_request(&ctx.client, json!({"iZoneV2Request": {"Type": kind, "No": no, "No1": 0}}))
        .map_err(device_err)?;
    v.get(key).cloned().ok_or_else(|| device_err(format!("Unexpected controller response (no {})", key)))
}

/// Send commands in order; stop at the first failure and say what was already applied.
fn send_all(ctx: &Ctx, commands: &[Value]) -> Result<(), (u16, String)> {
    for (n, cmd) in commands.iter().enumerate() {
        if let Err(e) = try_command_request(&ctx.client, cmd.clone()) {
            let note = if n > 0 { format!(" ({} earlier change(s) were applied)", n) } else { String::new() };
            return Err(device_err(format!("{}{}", e, note)));
        }
    }
    Ok(())
}

fn system(ctx: &Ctx) -> Result<Value, (u16, String)> {
    let mut sys = query(ctx, 1, 0, "SystemV2")?;
    let raw_warning = sys.get("Warnings").and_then(Value::as_str).unwrap_or("none").to_string();
    let steady = {
        let mut w = ctx.warnings.lock().unwrap_or_else(|p| p.into_inner());
        w.record(&raw_warning);
        w.steady()
    };
    if let Some(obj) = sys.as_object_mut() {
        for f in HIDDEN_SYSTEM_FIELDS {
            obj.remove(f);
        }
        // What the UI shows: only a warning that held steady over the sampling window.
        obj.insert("WarningSteady".to_string(), json!(steady));
    }
    Ok(sys)
}

fn num(v: &Value, key: &str) -> i64 {
    v.get(key).and_then(Value::as_i64).unwrap_or(0)
}

fn zone_count(sys: &Value) -> u8 {
    (num(sys, "NoOfZones").clamp(0, MAX_ZONES as i64)) as u8
}

fn get_system(ctx: &Ctx) -> ApiResult {
    system(ctx)
}

fn get_zones(ctx: &Ctx) -> ApiResult {
    let sys = system(ctx)?;
    let mut zones = Vec::new();
    for i in 0..zone_count(&sys) {
        zones.push(query(ctx, 2, i, "ZonesV2")?);
    }
    Ok(Value::Array(zones))
}

fn get_schedules(ctx: &Ctx) -> ApiResult {
    let mut list = Vec::new();
    for i in 0..MAX_SCHEDULES {
        list.push(query(ctx, 3, i, "SchedulesV2")?);
    }
    Ok(Value::Array(list))
}

// ----------------------------------------------------------------------------
// Validation helpers (same rules as the CLI)
// ----------------------------------------------------------------------------

/// °C (e.g. 22.5) -> raw x100, 15.0-30.0 in 0.5 steps.
fn temp_raw(v: &Value, what: &str) -> Result<u32, (u16, String)> {
    let t = v.as_f64().ok_or_else(|| bad(format!("{} must be a number", what)))?;
    let raw = (t * 100.0).round();
    if !t.is_finite() || !(1500.0..=3000.0).contains(&raw) || (raw as u32) % 50 != 0 {
        return Err(bad(format!("{} must be 15.0-30.0°C in 0.5°C steps", what)));
    }
    Ok(raw as u32)
}

fn as_bool(v: &Value, what: &str) -> Result<bool, (u16, String)> {
    match v {
        Value::Bool(b) => Ok(*b),
        Value::Number(n) if n.as_i64() == Some(0) => Ok(false),
        Value::Number(n) if n.as_i64() == Some(1) => Ok(true),
        _ => Err(bad(format!("{} must be true or false", what))),
    }
}

fn as_int(v: &Value, what: &str) -> Result<i64, (u16, String)> {
    v.as_i64().ok_or_else(|| bad(format!("{} must be a whole number", what)))
}

fn sys_mode(name: &str) -> Result<u8, (u16, String)> {
    match name.to_lowercase().as_str() {
        "cool" => Ok(1),
        "heat" => Ok(2),
        "vent" => Ok(3),
        "dry" => Ok(4),
        "auto" => Ok(5),
        _ => Err(bad("mode must be one of: cool, heat, vent, dry, auto")),
    }
}

fn sys_fan(name: &str, allow_nongasheat: bool) -> Result<u8, (u16, String)> {
    match name.to_lowercase().as_str() {
        "low" => Ok(1),
        "medium" => Ok(2),
        "high" => Ok(3),
        "auto" => Ok(4),
        "top" => Ok(5),
        "nongasheat" if allow_nongasheat => Ok(99),
        _ => Err(bad("fan must be one of: low, medium, high, auto, top")),
    }
}

fn zone_mode(name: &str) -> Result<u8, (u16, String)> {
    match name.to_lowercase().as_str() {
        "open" => Ok(1),
        "close" | "closed" | "off" => Ok(2),
        "auto" | "climate" => Ok(3),
        "override" => Ok(4),
        "constant" => Ok(5),
        _ => Err(bad("zone mode must be one of: auto, open, close, override, constant")),
    }
}

fn name_value(v: &Value, what: &str) -> Result<String, (u16, String)> {
    let s = v.as_str().ok_or_else(|| bad(format!("{} must be text", what)))?.trim().to_string();
    if s.is_empty() {
        return Err(bad(format!("{} cannot be empty", what)));
    }
    if s.len() > 15 {
        return Err(bad(format!("{} cannot exceed 15 characters", what)));
    }
    Ok(s)
}

fn percent_step5(v: &Value, what: &str) -> Result<u8, (u16, String)> {
    let p = as_int(v, what)?;
    if !(0..=100).contains(&p) || p % 5 != 0 {
        return Err(bad(format!("{} must be 0-100 in steps of 5", what)));
    }
    Ok(p as u8)
}

/// "HH:MM" -> (h, m); null -> not set (31, 63).
fn sched_time(v: &Value, what: &str) -> Result<(u8, u8), (u16, String)> {
    if v.is_null() {
        return Ok((31, 63));
    }
    let s = v.as_str().ok_or_else(|| bad(format!("{} must be HH:MM or null", what)))?;
    let parsed = s.split_once(':').and_then(|(h, m)| {
        if m.len() != 2 {
            return None;
        }
        Some((h.parse::<u8>().ok()?, m.parse::<u8>().ok()?))
    });
    match parsed {
        Some((h, m)) if h <= 23 && m <= 59 => Ok((h, m)),
        _ => Err(bad(format!("{} must be a valid time HH:MM", what))),
    }
}

fn valid_time(h: Option<i64>, m: Option<i64>) -> (i64, i64) {
    match (h, m) {
        (Some(h), Some(m)) if (0..=23).contains(&h) && (0..=59).contains(&m) => (h, m),
        _ => (31, 63),
    }
}

// ----------------------------------------------------------------------------
// Writes
// ----------------------------------------------------------------------------

fn post_system(ctx: &Ctx, b: &Value) -> ApiResult {
    let mut cmds = Vec::new();
    let mut turn_on_for_timer = false;
    if let Some(v) = b.get("power") {
        cmds.push(json!({"SysOn": as_bool(v, "power")? as u8}));
    }
    if let Some(v) = b.get("mode") {
        cmds.push(json!({"SysMode": sys_mode(v.as_str().unwrap_or(""))?}));
    }
    if let Some(v) = b.get("fan") {
        cmds.push(json!({"SysFan": sys_fan(v.as_str().unwrap_or(""), false)?}));
    }
    if let Some(v) = b.get("setpoint") {
        cmds.push(json!({"SysSetpoint": temp_raw(v, "Setpoint")?}));
    }
    if let Some(v) = b.get("sleep_timer") {
        let m = v.as_u64().filter(|m| SLEEP_TIMER_MINUTES.contains(m)).ok_or_else(|| {
            bad("sleep_timer must be one of 0, 30, 60, 120, 240, 360, 480, 720 minutes")
        })?;
        // Like the CLI: a timer only makes sense with the AC on.
        turn_on_for_timer = m > 0;
        cmds.push(json!({"SysSleepTimer": m}));
    }
    if cmds.is_empty() {
        return Err(bad("Nothing to change"));
    }
    if turn_on_for_timer && b.get("power").is_none() && num(&system(ctx)?, "SysOn") == 0 {
        cmds.insert(0, json!({"SysOn": 1}));
    }
    send_all(ctx, &cmds)?;
    get_system(ctx)
}

fn post_zone(ctx: &Ctx, index: u8, b: &Value) -> ApiResult {
    let sys = system(ctx)?;
    if index >= zone_count(&sys) {
        return Err(bad(format!("Zone index must be 0-{}", zone_count(&sys).saturating_sub(1))));
    }
    let mut cmds = Vec::new();
    if let Some(v) = b.get("name") {
        cmds.push(json!({"ZoneName": {"Index": index, "Name": name_value(v, "Zone name")?}}));
    }
    if let Some(v) = b.get("mode") {
        cmds.push(json!({"ZoneMode": {"Index": index, "Mode": zone_mode(v.as_str().unwrap_or(""))?}}));
    }
    if let Some(v) = b.get("setpoint") {
        let raw = temp_raw(v, "Setpoint")?;
        if num(&sys, "EcoLock") == 1 {
            let (lo, hi) = (num(&sys, "EcoMin"), num(&sys, "EcoMax"));
            if (raw as i64) < lo || (raw as i64) > hi {
                return Err(bad(format!(
                    "Economy lock is on: setpoint must be {:.1}-{:.1}°C",
                    lo as f64 / 100.0,
                    hi as f64 / 100.0
                )));
            }
        }
        cmds.push(json!({"ZoneSetpoint": {"Index": index, "Setpoint": raw}}));
    }
    if num(&sys, "AirflowLock") == 1 && (b.get("max_air").is_some() || b.get("min_air").is_some()) {
        return Err(bad("Airflow adjustment is locked. Turn off \"Airflow lock\" in Settings first."));
    }
    if num(&sys, "AirflowMinLock") == 1 && b.get("min_air").is_some() {
        return Err(bad("Minimum airflow adjustment is locked. Turn off \"Minimum airflow lock\" in Settings first."));
    }
    let max = b.get("max_air").map(|v| percent_step5(v, "Max air")).transpose()?;
    let min = b.get("min_air").map(|v| percent_step5(v, "Min air")).transpose()?;
    if let (Some(lo), Some(hi)) = (min, max) {
        if lo > hi {
            return Err(bad("Min air cannot be above max air"));
        }
    }
    if let Some(m) = max {
        cmds.push(json!({"ZoneMaxAir": {"Index": index, "MaxAir": m}}));
    }
    if let Some(m) = min {
        cmds.push(json!({"ZoneMinAir": {"Index": index, "MinAir": m}}));
    }
    if cmds.is_empty() {
        return Err(bad("Nothing to change"));
    }
    for cmd in &cmds {
        send_all(ctx, std::slice::from_ref(cmd))?;
        if cmd.get("ZoneSetpoint").is_some() {
            // The CLI sends setpoints twice; the controller sometimes ignores the first.
            thread::sleep(Duration::from_millis(100));
            send_all(ctx, std::slice::from_ref(cmd))?;
        }
    }
    query(ctx, 2, index, "ZonesV2")
}

fn day_flags(sched: &Value) -> Map<String, Value> {
    let mut m = Map::new();
    for d in ["M", "Tu", "W", "Th", "F", "Sa", "Su"] {
        let on = sched.get("DaysEnabled").and_then(|x| x.get(d)).and_then(Value::as_i64).unwrap_or(0);
        m.insert(d.to_string(), json!(if on == 1 { 1 } else { 0 }));
    }
    m
}

fn post_schedule(ctx: &Ctx, index: u8, b: &Value) -> ApiResult {
    let current = query(ctx, 3, index, "SchedulesV2")?;
    let mut cmds = Vec::new();

    if let Some(v) = b.get("name") {
        cmds.push(json!({"SchedName": {"Index": index, "Name": name_value(v, "Schedule name")?}}));
    }

    // Times and days share one command (SchedSettings), so merge with the current values.
    if b.get("start").is_some() || b.get("stop").is_some() || b.get("days").is_some() {
        let cur_start = valid_time(current.get("StartH").and_then(Value::as_i64), current.get("StartM").and_then(Value::as_i64));
        let cur_stop = valid_time(current.get("StopH").and_then(Value::as_i64), current.get("StopM").and_then(Value::as_i64));
        let (sh, sm) = match b.get("start") {
            Some(v) => { let (h, m) = sched_time(v, "Start time")?; (h as i64, m as i64) }
            None => cur_start,
        };
        let (eh, em) = match b.get("stop") {
            Some(v) => { let (h, m) = sched_time(v, "Stop time")?; (h as i64, m as i64) }
            None => cur_stop,
        };
        let days = match b.get("days") {
            Some(v) => {
                let list = v.as_array().ok_or_else(|| bad("days must be a list, e.g. [\"Mon\", \"Tue\"]"))?;
                let mut m = Map::new();
                for d in ["M", "Tu", "W", "Th", "F", "Sa", "Su"] {
                    m.insert(d.to_string(), json!(0));
                }
                for d in list {
                    let key = match d.as_str().unwrap_or("").to_lowercase().as_str() {
                        "mon" => "M", "tue" => "Tu", "wed" => "W", "thu" => "Th",
                        "fri" => "F", "sat" => "Sa", "sun" => "Su",
                        other => return Err(bad(format!("Unknown day '{}'. Use Mon Tue Wed Thu Fri Sat Sun", other))),
                    };
                    m.insert(key.to_string(), json!(1));
                }
                m
            }
            None => day_flags(&current),
        };
        cmds.push(json!({"SchedSettings": {
            "Index": index, "StartH": sh, "StartM": sm, "StopH": eh, "StopM": em, "DaysEnabled": days
        }}));
    }

    if let Some(v) = b.get("mode") {
        cmds.push(json!({"SchedAcMode": {"Index": index, "Mode": sys_mode(v.as_str().unwrap_or(""))?}}));
    }
    if let Some(v) = b.get("fan") {
        cmds.push(json!({"SchedAcFan": {"Index": index, "Fan": sys_fan(v.as_str().unwrap_or(""), true)?}}));
    }

    // Zones: full positional array, starting from the current one.
    if let Some(v) = b.get("zones") {
        let mut zones: Vec<(i64, i64)> = current
            .get("Zones")
            .and_then(Value::as_array)
            .map(|a| a.iter().map(|z| (num(z, "Mode"), num(z, "Setpoint"))).collect())
            .unwrap_or_default();
        if zones.is_empty() {
            return Err(device_err("Schedule returned no zone settings to update"));
        }
        let list = v.as_array().ok_or_else(|| bad("zones must be a list"))?;
        for z in list {
            let i = z.get("index").and_then(Value::as_u64).ok_or_else(|| bad("Each zone needs an index"))? as usize;
            if i >= zones.len() {
                return Err(bad(format!("Zone index {} is out of range", i)));
            }
            let mode = zone_mode(z.get("mode").and_then(Value::as_str).unwrap_or(""))?;
            let sp = temp_raw(z.get("setpoint").unwrap_or(&Value::Null), "Zone setpoint")?;
            zones[i] = (mode as i64, sp as i64);
        }
        let arr: Vec<Value> = zones.iter().map(|(m, s)| json!({"Mode": m, "Setpoint": s})).collect();
        cmds.push(json!({"SchedZones": {"Index": index, "Zones": arr}}));
    }

    // Enable/disable last (after times are set); skip if unchanged, since the controller
    // rejects SchedEnable for a schedule that isn't set up.
    if let Some(v) = b.get("enabled") {
        let want = as_bool(v, "enabled")?;
        if want != (num(&current, "Enabled") == 1) {
            cmds.push(json!({"SchedEnable": {"Index": index, "Enabled": want as u8}}));
        }
    }

    if cmds.is_empty() {
        return query(ctx, 3, index, "SchedulesV2");
    }
    send_all(ctx, &cmds)?;
    query(ctx, 3, index, "SchedulesV2")
}

fn clear_schedule(ctx: &Ctx, index: u8) -> ApiResult {
    let current = query(ctx, 3, index, "SchedulesV2")?;
    let mut cmds = Vec::new();
    if num(&current, "Enabled") == 1 {
        cmds.push(json!({"SchedEnable": {"Index": index, "Enabled": 0}}));
    }
    let start = valid_time(current.get("StartH").and_then(Value::as_i64), current.get("StartM").and_then(Value::as_i64));
    let stop = valid_time(current.get("StopH").and_then(Value::as_i64), current.get("StopM").and_then(Value::as_i64));
    let any_day = day_flags(&current).values().any(|v| v.as_i64() == Some(1));
    if start != (31, 63) || stop != (31, 63) || any_day {
        cmds.push(json!({"SchedSettings": {
            "Index": index, "StartH": 31, "StartM": 63, "StopH": 31, "StopM": 63,
            "DaysEnabled": {"M": 0, "Tu": 0, "W": 0, "Th": 0, "F": 0, "Sa": 0, "Su": 0}
        }}));
    }
    if current.get("Name").and_then(Value::as_str).map(|s| !s.is_empty()).unwrap_or(false) {
        cmds.push(json!({"SchedName": {"Index": index, "Name": ""}}));
    }
    send_all(ctx, &cmds)?;
    query(ctx, 3, index, "SchedulesV2")
}

// ----------------------------------------------------------------------------
// Settings: one table drives both validation (here) and the UI's forms.
// ----------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Kind {
    Bool,
    Number { min: i64, max: i64, step: i64 },
    Select(&'static [(i64, &'static str)]),
    Action(&'static str),
}

struct Setting {
    key: &'static str,      // iZone command key
    status: &'static str,   // where the current value is in SystemV2 ("A" or "A.B")
    section: &'static str,
    label: &'static str,
    help: &'static str,
    kind: Kind,
    scale: f64,             // displayed value = raw / scale
    unit: &'static str,
}

const S_COMFORT: &str = "Comfort & economy";
const S_AIRFLOW: &str = "Airflow & dampers";
const S_MAINT: &str = "Maintenance";
const S_VENT: &str = "Ventilation";
const S_CB: &str = "Coolbreeze";

const fn n(min: i64, max: i64, step: i64) -> Kind {
    Kind::Number { min, max, step }
}

const SETTINGS: &[Setting] = &[
    Setting { key: "EconomyLock", status: "EcoLock", section: S_COMFORT, label: "Economy lock", help: "Limit all setpoints to the range below", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "EconomyMin", status: "EcoMin", section: S_COMFORT, label: "Economy minimum", help: "Lowest setpoint allowed while economy lock is on", kind: n(1500, 3000, 50), scale: 100.0, unit: "°C" },
    Setting { key: "EconomyMax", status: "EcoMax", section: S_COMFORT, label: "Economy maximum", help: "Highest setpoint allowed while economy lock is on", kind: n(1500, 3000, 50), scale: 100.0, unit: "°C" },
    Setting { key: "AutoModeDeadB", status: "AutoModeDeadB", section: S_COMFORT, label: "Auto mode deadband", help: "How far from the setpoint before Auto switches between heating and cooling", kind: n(75, 500, 5), scale: 100.0, unit: "°C" },
    Setting { key: "ScroogeMode", status: "Scrooge", section: S_COMFORT, label: "Scrooge mode", help: "Energy-saving mode", kind: Kind::Bool, scale: 1.0, unit: "" },

    Setting { key: "AirflowLock", status: "AirflowLock", section: S_AIRFLOW, label: "Airflow lock", help: "Prevent changes to zone min and max airflow", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "AirflowMinLock", status: "AirflowMinLock", section: S_AIRFLOW, label: "Minimum airflow lock", help: "Prevent changes to zone minimum airflow only", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "DamperTime", status: "DamperTime", section: S_AIRFLOW, label: "Damper time", help: "Damper motor run time; 0 = automatic", kind: n(0, 255, 1), scale: 1.0, unit: "s" },
    Setting { key: "OpenDampersWhenOff", status: "OpenDampersWhenOff", section: S_AIRFLOW, label: "Open dampers when off", help: "Open all dampers when the AC is turned off", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "ReverseDampers", status: "ReverseDampers", section: S_AIRFLOW, label: "Reverse dampers", help: "For dampers wired in reverse", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "StaticP", status: "StaticP", section: S_AIRFLOW, label: "Static pressure", help: "Midea-type units: 0 lowest to 4 highest", kind: n(0, 4, 1), scale: 1.0, unit: "" },
    Setting { key: "CnstCtrlAreaEn", status: "CnstCtrlAreaEn", section: S_AIRFLOW, label: "Constant control by area", help: "Size constant zones by floor area", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "CnstCtrlArea", status: "CnstCtrlArea", section: S_AIRFLOW, label: "Constant control area", help: "Area to be covered by the constant zone", kind: n(1, 1000, 1), scale: 1.0, unit: "m²" },

    Setting { key: "FilterWarn", status: "FilterWarn", section: S_MAINT, label: "Filter reminder", help: "Remind to clean the filter", kind: Kind::Select(&[(0, "Off"), (3, "Every 3 months"), (6, "Every 6 months"), (12, "Every 12 months")]), scale: 1.0, unit: "" },
    Setting { key: "ResetWarning", status: "Warnings", section: S_MAINT, label: "Filter warning", help: "Clear the filter warning after cleaning", kind: Kind::Action("filter"), scale: 1.0, unit: "" },

    Setting { key: "VentilationRfSetpoint", status: "Ventilation.RhSetpoint", section: S_VENT, label: "Humidity setpoint", help: "", kind: n(5, 95, 1), scale: 1.0, unit: "%" },
    Setting { key: "VentilationVocsSetpoint", status: "Ventilation.VocsSetpoint", section: S_VENT, label: "VOC setpoint", help: "", kind: n(50, 2500, 10), scale: 1.0, unit: "ppb" },
    Setting { key: "VentilationEco2Setpoint", status: "Ventilation.Eco2Setpoint", section: S_VENT, label: "eCO₂ setpoint", help: "", kind: n(500, 1500, 10), scale: 1.0, unit: "ppm" },
    Setting { key: "VentilationFanStageDelay", status: "Ventilation.FanStageDelay", section: S_VENT, label: "Fan stage delay", help: "", kind: n(3, 240, 1), scale: 1.0, unit: "min" },
    Setting { key: "VentilationCycleFanOff", status: "Ventilation.CycleFanOff", section: S_VENT, label: "Cycle fan off", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "VentilationUseRhControl", status: "Ventilation.UseRhControl", section: S_VENT, label: "Humidity control", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "VentilationUseVcosControl", status: "Ventilation.UseVcosControl", section: S_VENT, label: "VOC control", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "VentilationUseEco2Control", status: "Ventilation.UseEco2Control", section: S_VENT, label: "eCO₂ control", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },

    Setting { key: "CoolbreezeFanSpeed", status: "Coolbreeze.FanSpeed", section: S_CB, label: "Fan speed", help: "", kind: n(1, 100, 1), scale: 1.0, unit: "%" },
    Setting { key: "CoolbreezeRhSetpoint", status: "Coolbreeze.RhSet", section: S_CB, label: "Humidity setpoint", help: "", kind: n(10, 90, 1), scale: 1.0, unit: "%" },
    Setting { key: "CoolbreezePrewEn", status: "Coolbreeze.PrewEn", section: S_CB, label: "Pre-wash", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "CoolbreezePrewTime", status: "Coolbreeze.PrewTime", section: S_CB, label: "Pre-wash time", help: "", kind: n(1, 60, 1), scale: 1.0, unit: "min" },
    Setting { key: "CoolbreezeDrAfPrewEn", status: "Coolbreeze.DrAfPrewEn", section: S_CB, label: "Drain after pre-wash", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "CoolbreezeDrCycEn", status: "Coolbreeze.DrCycEn", section: S_CB, label: "Drain cycle", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "CoolbreezeDrCycPer", status: "Coolbreeze.DrCycPer", section: S_CB, label: "Drain cycle period", help: "", kind: n(60, 3000, 60), scale: 60.0, unit: "h" },
    Setting { key: "CoolbreezePostwEn", status: "Coolbreeze.PostwEn", section: S_CB, label: "Post-wash", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "CoolbreezeDrBfPostwEn", status: "Coolbreeze.DrBfPostwEn", section: S_CB, label: "Drain before post-wash", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "CoolbreezePostwT", status: "Coolbreeze.PostwT", section: S_CB, label: "Post-wash time", help: "", kind: n(5, 30, 1), scale: 1.0, unit: "min" },
    Setting { key: "CoolbreezeInverter", status: "Coolbreeze.Inverter", section: S_CB, label: "Inverter", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "CoolbreezeResumeLast", status: "Coolbreeze.ResumeLast", section: S_CB, label: "Resume last state", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "CoolbreezeFanMaxAuto", status: "Coolbreeze.FanMaxAuto", section: S_CB, label: "Max fan (auto)", help: "", kind: n(1, 100, 1), scale: 1.0, unit: "%" },
    Setting { key: "CoolbreezeFanMax", status: "Coolbreeze.FanMax", section: S_CB, label: "Max fan (manual)", help: "", kind: n(1, 100, 1), scale: 1.0, unit: "%" },
    Setting { key: "CoolbreezeExhMax", status: "Coolbreeze.ExhMax", section: S_CB, label: "Max fan (exhaust)", help: "", kind: n(1, 100, 1), scale: 1.0, unit: "%" },
    Setting { key: "CoolbreezeExhEn", status: "Coolbreeze.ExhEn", section: S_CB, label: "Exhaust mode", help: "", kind: Kind::Bool, scale: 1.0, unit: "" },
    Setting { key: "CoolbreezeCtrlSens", status: "Coolbreeze.CtrlSens", section: S_CB, label: "Control sensor", help: "", kind: Kind::Select(&[(0, "Screen"), (1, "Remote")]), scale: 1.0, unit: "" },
    Setting { key: "CoolbreezeCalibTemp", status: "Coolbreeze.CalibTemp", section: S_CB, label: "Temperature calibration", help: "", kind: n(-50, 50, 1), scale: 10.0, unit: "°C" },
    Setting { key: "CoolbreezeDeadTemp", status: "Coolbreeze.DeadTemp", section: S_CB, label: "Temperature deadband", help: "", kind: n(100, 500, 10), scale: 100.0, unit: "°C" },
    Setting { key: "CoolbreezeAutoFanMaxTime", status: "Coolbreeze.AutoFanMaxTime", section: S_CB, label: "Auto fan max-speed time", help: "", kind: n(0, 60, 1), scale: 1.0, unit: "min" },
];

fn status_value(sys: &Value, path: &str) -> Value {
    path.split('.').fold(sys.clone(), |v, k| v.get(k).cloned().unwrap_or(Value::Null))
}

/// Ventilation settings only for ventilation systems (universal controller type 11);
/// Coolbreeze only when the controller reports a Coolbreeze unit.
fn section_visible(sys: &Value, section: &str) -> bool {
    match section {
        S_VENT => sys.get("GasHeat").and_then(|g| g.get("Type")).and_then(Value::as_i64) == Some(11),
        S_CB => sys.get("Coolbreeze").is_some(),
        _ => true,
    }
}

fn get_settings(ctx: &Ctx) -> ApiResult {
    let sys = system(ctx)?;
    let mut sections: Vec<(String, Vec<Value>)> = Vec::new();
    for s in SETTINGS.iter().filter(|s| section_visible(&sys, s.section)) {
        let kind = match s.kind {
            Kind::Bool => json!({"type": "bool"}),
            Kind::Number { min, max, step } => json!({"type": "number", "min": min, "max": max, "step": step}),
            Kind::Select(opts) => json!({"type": "select", "options": opts.iter().map(|(v, l)| json!({"value": v, "label": l})).collect::<Vec<_>>()}),
            Kind::Action(_) => json!({"type": "action"}),
        };
        let item = json!({
            "key": s.key, "label": s.label, "help": s.help, "kind": kind,
            "scale": s.scale, "unit": s.unit,
            "value": if s.key == "ResetWarning" { status_value(&sys, "WarningSteady") } else { status_value(&sys, s.status) },
        });
        match sections.iter_mut().find(|(t, _)| t == s.section) {
            Some((_, items)) => items.push(item),
            None => sections.push((s.section.to_string(), vec![item])),
        }
    }
    Ok(Value::Array(sections.into_iter().map(|(t, items)| json!({"title": t, "items": items})).collect()))
}

fn post_setting(ctx: &Ctx, b: &Value) -> ApiResult {
    let key = b.get("key").and_then(Value::as_str).ok_or_else(|| bad("key is required"))?;
    let s = SETTINGS.iter().find(|s| s.key == key).ok_or_else(|| bad(format!("Unknown setting '{}'", key)))?;
    let v = b.get("value").unwrap_or(&Value::Null);
    let value: Value = match s.kind {
        Kind::Bool => json!(as_bool(v, s.label)? as u8),
        Kind::Number { min, max, step } => {
            let x = as_int(v, s.label)?;
            if x < min || x > max || (x - min) % step != 0 {
                let f = |r: i64| {
                    let d = r as f64 / s.scale;
                    if s.scale == 1.0 { format!("{}", r) } else { format!("{}", d) }
                };
                let step_txt = if s.scale == 1.0 { format!("{}", step) } else { format!("{}", step as f64 / s.scale) };
                return Err(bad(format!("{} must be {}-{}{} in steps of {}", s.label, f(min), f(max), s.unit, step_txt)));
            }
            json!(x)
        }
        Kind::Select(opts) => {
            let x = as_int(v, s.label)?;
            if !opts.iter().any(|(o, _)| *o == x) {
                return Err(bad(format!("{} has no option {}", s.label, x)));
            }
            json!(x)
        }
        Kind::Action(arg) => json!(arg),
    };
    // Keep economy min <= max.
    if key == "EconomyMin" || key == "EconomyMax" {
        let sys = system(ctx)?;
        let x = value.as_i64().unwrap_or(0);
        if key == "EconomyMin" && x > num(&sys, "EcoMax") {
            return Err(bad("Economy minimum cannot be above the maximum"));
        }
        if key == "EconomyMax" && x < num(&sys, "EcoMin") {
            return Err(bad("Economy maximum cannot be below the minimum"));
        }
    }
    let mut cmd = Map::new();
    cmd.insert(key.to_string(), value);
    // Note: some controllers keep flickering `Warnings` after a reset, so a reset can't be
    // confirmed here; the UI says "sent". Start the steady-warning window afresh.
    send_all(ctx, &[Value::Object(cmd)])?;
    if key == "ResetWarning" {
        ctx.warnings.lock().unwrap_or_else(|p| p.into_inner()).samples.clear();
    }
    get_settings(ctx)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Feed (seconds, warning) samples and return the steady warning at the last sample's time.
    fn steady_after(samples: &[(u64, &str)]) -> String {
        let t0 = Instant::now();
        let mut w = WarningSamples::default();
        for (s, v) in samples {
            w.record_at(t0 + Duration::from_secs(*s), v);
        }
        w.steady_at(t0 + Duration::from_secs(samples.last().unwrap().0))
    }

    #[test]
    fn flicker_is_not_a_warning() {
        // Pattern seen on a real controller: filter/none alternating every few seconds.
        let s: Vec<(u64, &str)> = (0..=60).step_by(5).map(|t| (t, if (t / 5) % 3 == 0 { "none" } else { "filter" })).collect();
        assert_eq!(steady_after(&s), "none");
    }

    #[test]
    fn steady_warning_is_reported() {
        let s: Vec<(u64, &str)> = (0..=35).step_by(5).map(|t| (t, "filter")).collect();
        assert_eq!(steady_after(&s), "filter");
    }

    #[test]
    fn needs_a_full_window_before_reporting() {
        // 15 s of "filter" (like the longest flicker stretch) is not enough.
        let s: Vec<(u64, &str)> = (0..=15).step_by(5).map(|t| (t, "filter")).collect();
        assert_eq!(steady_after(&s), "none");
        // A single "none" inside the window cancels it.
        let mut s: Vec<(u64, &str)> = (0..=35).step_by(5).map(|t| (t, "filter")).collect();
        s[4].1 = "none";
        assert_eq!(steady_after(&s), "none");
    }

    #[test]
    fn no_warning_and_empty() {
        let s: Vec<(u64, &str)> = (0..=35).step_by(5).map(|t| (t, "none")).collect();
        assert_eq!(steady_after(&s), "none");
        assert_eq!(WarningSamples::default().steady(), "none");
    }
}
