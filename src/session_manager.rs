use rand::{Rng, distributions::Alphanumeric};
use std::collections::HashMap;

#[derive(Debug, Clone)]
/// In-memory session object.
///
/// Why: Track per-client state via a cookie (`session_id`).
/// How: Stores a random `id` and a key-value map for extensible data.
pub struct Session {
    pub id: String,
    pub data: HashMap<String, String>,
}

#[derive(Debug, Clone)]
/// Session manager holding all active sessions.
///
/// Why: Provide lookups and creation of sessions from incoming cookies.
/// How: Map from `session_id` to `Session` in memory.
pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    /// Construct an empty `SessionManager`.
    ///
    /// Why: Initialize session storage at server startup.
    /// How: Creates an empty `HashMap`.
    pub fn new() -> Self {
        SessionManager {
            sessions: HashMap::new(),
        }
    }

    /// Generate a random 32-character `session_id`.
    ///
    /// Why: Use an unpredictable identifier for client sessions.
    /// How: Samples alphanumeric characters from a thread-local RNG.
    fn generate_session_id() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(32)
            .map(char::from)
            .collect()
    }
    /// Get or create a session based on the `Cookie` header.
    ///
    /// Why: Reuse valid sessions; otherwise create a new one and return it.
    /// How: Extracts `session_id` from cookies; looks it up in the internal map;
    /// if missing, generates a new session and stores it.
    pub fn get_or_create_session(&mut self, cookie_header: Option<&str>) -> Session {
        if let Some(cookie_header) = cookie_header {
            if let Some(session_id) = Self::extract_session_id(cookie_header) {
                println!("🔑 Found session_id in cookies: {}", session_id);
                if let Some(session) = self.sessions.get(&session_id) {
                    println!("✅ Reusing existing session: {}", session_id);
                    return session.clone();
                } else {
                    println!("❌ Session ID not found in session store");
                }
            } else {
                println!("❌ No session_id found in cookies");
            }
        } else {
            println!("📭 No Cookie header received");
        }

        // create a new session if no valid session_id is found
        let new_id = Self::generate_session_id();
        let new_session = Session {
            id: new_id.clone(),
            data: HashMap::new(),
        };

        self.sessions.insert(new_id.clone(), new_session.clone());
        new_session
    }

    /// Extract `session_id` from the `Cookie` header if present.
    ///
    /// Why: Helpers keep `get_or_create_session` minimal and readable.
    /// How: Splits cookies on `;` and looks for the `session_id=` prefix.
    fn extract_session_id(cookie_header: &str) -> Option<String> {
        for cookie in cookie_header.split(';') {
            let cookie = cookie.trim();
            if cookie.starts_with("session_id=") {
                return Some(cookie.trim_start_matches("session_id=").to_string());
            }
        }
        None
    }
}
