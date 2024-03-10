# CRIM | A Rust IM 🦀
I've designed and worked on CRIM as a means to strengthen my understanding of Rust as a language, and to better familiarize myself with lower-level concepts, cybersecurity, and database management. I continually work to improve it and make it as secure as I can.

## Database
CRIM uses mongoDB to store data externally, but can likely be refactored to use other databases so as long as they can be converted to a BSON format. Database details can be set in the `.env` file.

## Encryption
### Login
Passwords are encrypted with the typical salting method; A salt is generated, added to the password, hashed using Argon2, encoded with base64, and then sent to an external mongoDB database:
> From [`login.rs`](src/core/login.rs)
```rust
// get user input

let mut salt: [u8; 256] = [0; 256];
getrandom(&mut salt).expect("Failed to generate random salt.");
let mut output: [u8; 256] = [0u8; 256];
Argon2::default().hash_password_into(&password.into_bytes(), &salt, &mut output).expect("failed to hash password");
let base64_encoded = general_purpose::STANDARD.encode(&output);
let new_profile: Profile = Profile { username: username, hash: base64_encoded, salt: salt.to_vec() };

// database actions below
```
Due to project constraints, I cannot set up a dedicated backend, so authentication is not very secure; ideally, user authentication would occur serverside.
However, for this specific project, it's not a major concern because with E2EE, messages would not be readable either way without the private key, which is encrypted
with the user's password.
For future projects though, I may need to work in a backend to be more secure with user auth.

### E2EE with Messaging
Wip :)
