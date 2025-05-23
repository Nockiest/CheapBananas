# Backend for CheapBananas

This is the backend service for the CheapBananas application. It is built using Rust and the Axum framework, with PostgreSQL as the database.

## Features

- **Product Management**: CRUD operations for products.
- **Product Entry Management**: Manage product entries with details like price, volume, and shop.
- **Shop Management**: CRUD operations for shops.
- **Validation**: Ensures data integrity and proper deserialization of JSON payloads.
- **Logging**: Debug logs to trace the flow of data.

## Project Structure

- `src/`
  - `app.rs`: Contains the main application logic and routes.
  - `db.rs`: Handles database interactions.
  - `models.rs`: Defines data models.
  - `utils/`: Utility functions.
- `migrations/`: Database migration files.
- `tests/`: Integration tests for the backend.

## Setup Instructions

### Prerequisites

- Rust and Cargo
- PostgreSQL

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/Nockiest/CheapBananas.git
   ```
2. Navigate to the backend directory:
   ```bash
   cd CheapBananas/backend
   ```
3. Install dependencies:
   ```bash
   cargo build
   ```

### Database Setup

1. Create a PostgreSQL database.
2. Apply migrations:
   ```bash
   cargo install sqlx-cli
   sqlx migrate run
   ```

### Running the Server

Start the backend server:
```bash
cargo run
```

The server will run on `http://localhost:4000` by default.

## Testing

Run the tests:
```bash
cargo test
```

## API Endpoints

### Products
- `GET /products`: Retrieve all products.
- `POST /products`: Add a new product.
- `PUT /products/{id}`: Update a product.
- `DELETE /products/{id}`: Delete a product.

### Product Entries
- `GET /product-entries`: Retrieve all product entries.
- `POST /product-entries`: Add a new product entry.
- `DELETE /product-entries/{id}`: Delete a product entry.

### Shops
- `GET /shops`: Retrieve all shops.
- `POST /shops`: Add a new shop.
- `DELETE /shops/{id}`: Delete a shop.

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
