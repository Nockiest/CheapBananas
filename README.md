# CheapBananas

CheapBananas is a web application designed to help users manage and compare product entries, shops, and related data efficiently. The project consists of a backend written in Rust and a frontend built with Next.js.

## Features

- **Product Management**: Add, update, and delete products.
- **Product Entry Management**: Manage product entries with details like price, volume, and shop.
- **Shop Management**: Add and manage shops.
- **Search and Suggestions**: Real-time search and suggestions for products and shops.
- **Data Validation**: Ensures data integrity with validations for required fields and duplicate entries.

## Project Structure

### Backend
- **Language**: Rust
- **Framework**: Axum
- **Database**: PostgreSQL
- **Directory**: `backend/`

Key files:
- `app.rs`: Contains the main application logic and routes.
- `db.rs`: Handles database interactions.
- `models.rs`: Defines data models.
- `utils/`: Utility functions.

### Frontend
- **Framework**: Next.js
- **Language**: TypeScript
- **Directory**: `cheap_bananas/`

Key files:
- `app/page.tsx`: Main page for managing products and entries.
- `app/compare/page.tsx`: Page for comparing product entries.
- `components/`: Reusable UI components.
- `hooks/`: Custom React hooks.
- `utils/`: Utility functions for the frontend.

## Setup Instructions

### Prerequisites
- Node.js and npm
- Rust and Cargo
- PostgreSQL

### Backend Setup
1. Navigate to the `backend/` directory:
   ```bash
   cd backend
   ```
2. Install dependencies:
   ```bash
   cargo build
   ```
3. Set up the database:
   - Create a PostgreSQL database.
   - Run migrations located in the `migrations/` directory.
4. Start the backend server:
   ```bash
   cargo run
   ```

### Frontend Setup
1. Navigate to the `cheap_bananas/` directory:
   ```bash
   cd cheap_bananas
   ```
2. Install dependencies:
   ```bash
   npm install
   ```
3. Start the development server:
   ```bash
   npm run dev
   ```

## Testing

### Backend Tests
1. Navigate to the `backend/` directory.
2. Run tests:
   ```bash
   cargo test
   ```

### Frontend Tests
1. Navigate to the `cheap_bananas/` directory.
2. Run Cypress tests:
   ```bash
   npx cypress open
   ```

## Contributing

Contributions are welcome! Please follow these steps:
1. Fork the repository.
2. Create a new branch for your feature or bugfix:
   ```bash
   git checkout -b feature-name
   ```
3. Commit your changes:
   ```bash
   git commit -m "Description of changes"
   ```
4. Push to your fork:
   ```bash
   git push origin feature-name
   ```
5. Open a pull request.

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Acknowledgments

- [Axum](https://github.com/tokio-rs/axum) for the backend framework.
- [Next.js](https://nextjs.org/) for the frontend framework.
- [Cypress](https://www.cypress.io/) for end-to-end testing.
