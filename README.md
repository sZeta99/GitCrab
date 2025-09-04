# GitCrab: Comprehensive Project Report

## Introduction

**GitCrab** is a web application developed in Rust with the Loco.rs framework (inspired by Ruby on Rails), designed for robust, secure, and efficient personal Git repository management in the style of GitHub. It leverages a strict **Model-View-Controller (MVC)** architecture, Docker containerization, PostgreSQL database, integrated SSH server, and JWT-based authentication, focusing on modularity and scalability.

---

![alt text](<Screenshot1.png>)

---

## Installation & Usage

### Prerequisites
- A system with Docker installed (Linux recommended, or WSL2 on Windows for best compatibility).
- Internet connection for first-time Docker image pulls.

### Installation Steps
1. Clone the repository to a directory of your choice.
2. Build the Docker image with `docker compose build`.
3. Start the application using `docker compose up`. Required images are downloaded automatically.

### How to Use
- Access via browser at `localhost:5051` (tested with Firefox).
- Register an account, [optional: view the welcome email via Mailpit (`localhost:8025`)].
- Log in and create repositories under "MyRepo".
![alt text](<Screenshot2.png>)
- Add your public SSH key via the SSH section.
![alt text](<Screenshot3.png>)
- Clone your repository using the provided link, make changes, and push or pull as needed.
- View repository changes and content inside the web explorer, available by clicking on the name of the repo in the "MyRepos" Section.
![alt text](<Screenshot4.png>)
- Logout via the dedicated UI button.

---

## Dedicated Section: MVC Pattern in GitCrab

### What is MVC?

The **Model-View-Controller** (MVC) pattern is a proven architectural approach for separating application logic, presentation, and data management. It enables modular code structure, easier maintenance, and a clear separation of concerns.

#### 1. **Model**
- Handles all data-related logic.
- In GitCrab, `Model` entities include `User`, `Repository`, and `SshKey`.
- Uses **SeaORM** for a type-safe, database-agnostic abstraction.
- Responsible for data validation, business logic, and synchronization between database records and filesystem entities (e.g., repositories, SSH keys).

**Example**:  
```rust
struct SshKey {
    id: i32,
    title: String,
    public_key: String,
    created_at: DateTime,
}
```
SeaORM enables high-level CRUD operations without direct SQL, ensuring safer logic.

#### 2. **View**
- Responsible for rendering HTML and presenting data to the user.
- Uses **Tera** templates to generate dynamic web pages.
- Receives data from Controllers and formats it for display.

**Example** (Tera template):
```tera
<table>
    <tr>
        <th>Title</th>
        <th>keys</th>
        <th>Create date</th>
    </tr>
    {% for key in ssh_keys %}
        <tr>
            <td>{{ key.title }}</td>
            <td>{{ key.public_key }}</td>
            <td>{{ key.created_at | date }}</td>
        </tr>
    {% endfor %}
</table>
```

#### 3. **Controller**
- Orchestrates requests, connecting Models and Views.
- Handles routing, validation, business logic, and serialization.
- Implements HTTP endpoints using Axum, passing data as required.

**Example** (Rust/Axum):
```rust
async fn add(
    State(ctx): State<AppContext>,
    Form(params): Form<SshKeyParams>,
) -> Result<Redirect> {
    let new_key = ActiveModel {
        title: Set(params.title),
        public_key: Set(params.public_key),
        ..Default::default()
    };
    let saved_key = new_key.insert(&ctx.db).await?;
    Ok(Redirect::to("/sshes"))
}
```

### Why MVC?
- **Separation of responsibilities**: Data, interface, and logic are decoupled.
- **Easier debugging and scaling**: Isolate issues and expand features faster.
- **Standard practices**: Encourages modularity and code reuse.

---

## Core Features

### 1. Authentication

- Secure user registration, login, and logout, all handled via JWT (JSON Web Tokens) and cookies.
- Upon registration, users receive a confirmation email (handled locally via Mailpit for development/testing).
- JWTs are stored as `HttpOnly` cookies to prevent XSS and CSRF attacks. Sessions stay valid as long as the token is.
- All authentication is enforced via dedicated Axum middleware.

**Limitation**: SSH access control is not user-specific (all registered users have repository SSH access).

### 2. Repository Management

- Fully-featured CRUD operations for repositories; all changes reflected on both the database (metadata) and on-disk with bare Git repos.
- Uses a custom `GitService` module for atomic operations, including rollback in case of errors.
- Repositories are sanitized for naming consistency, and only authenticated users can manipulate them.
- Users can explore repository content through a built-in explorer in the web interface.


### 3. SSH Key Management

- Users add, edit, or delete their public SSH keys via web UI, including validation.
- The `SshKeyService` manages the `authorized_keys` file atomically, combining database and filesystem changes.
- All SSH key actions are tracked and auditable via logs. Keys are validated and linked to users.
- Separate API endpoints enable programmatic or UI-based key management.




### 4. Integrated SSH Server

- The system runs an SSH daemon (`sshd`) in the container for native Git operations (`git pull`/`git push`).
- Connections use a system-wide `git` user with a restricted shell; all SSH commands route through a dispatch script, ensuring only Git-safe commands are allowed.
- Only valid repo and commands (`git-upload-pack`, `git-receive-pack`) are permitted.
- All operations, valid/invalid, are logged for auditing.

---

## Infrastructure, Docker Compose, and Data Layers

### Database: 
- Uses PostgreSQL (v16) for all persistent storage, managed as a separate Docker container with dedicated volumes for durability.

### Containerization:
- Managed through Docker Compose, which defines all services, environment variables, and volumes for a fully reproducible stack.
- Main services include:
    - `db`: PostgreSQL
    - `app`: GitCrab (web + SSH)
    - `mailer`: Mailpit for debug of mailing system
- All inter-service communication occurs over a private Docker network with shared persistent volumes for repositories and SSH keys.

### Volumes:
- `postgres_data`: Persistent DB storage.
- `git_repositories`: Storage for all git repositories.
- `git_ssh_keys`: Secure storage for SSH key operations.

---

## Server SSH: Architecture & Security

- SSH server is tightly locked-down, with all commands handled by a dispatch script via a restricted shell.
- Custom user (`git`) isolates privileges for all SSH-based operations.
- Permissions on directories/files are strictly controlled (`700/600` for sensitive files, `755` for repositories).
- All allowed commands are checked, other SSH access is denied.
- Full audit logs for all SSH activity.

---

## Security Considerations

- JWT and HttpOnly cookies is a resonably safe authentication method.
- CRUD operations protected by JWT-only access.
- All SSH and repository operations are event-logged.
- Names and keys validated and sanitized at every level.
- Atomic and rollback-safe file/database operations for maximum integrity.

---

## Limitations & Future Directions

- Individual SSH access control per repository is not yet implemented.
- Currently all SSH users with web access can connect to all repositories.
- Future improvements should include more granular SSH permissioning, advanced user management, finer audit logging and a more complete mailing system with password retrival.

---

## Conclusion

GitCrab implements a, modular, and highly secure solution for self-hosted Git repository management. By leveraging the MVC pattern, containerization, strong authentication, robust SSH integration, and clear separation of logic, presentation, and data, it ensures maintainability and extensibility for future needs. This architecture makes GitCrab a strong foundation for both individual users and teams seeking control over their own Git infrastructure.