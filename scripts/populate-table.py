import psycopg2

# PostgreSQL connection parameters
db_params = {
    "host": "localhost",
    "port": 5432,
    "user": "testuser",
    "password": "testpassword",
    "database": "testdb",
}


def create_table():
    # Connect to the PostgreSQL database
    connection = psycopg2.connect(**db_params)

    try:
        # Create a cursor object to execute SQL queries
        with connection.cursor() as cursor:
            # Execute a query to create the 'test' table if it doesn't exist
            cursor.execute(
                """
                CREATE TABLE IF NOT EXISTS test (
                    id SERIAL PRIMARY KEY,
                    data VARCHAR(255) NOT NULL
                );
                """
            )

        # Commit the changes
        connection.commit()

    finally:
        # Close the database connection
        connection.close()


def insert_data():
    # Connect to the PostgreSQL database
    connection = psycopg2.connect(**db_params)

    try:
        # Create a cursor object to execute SQL queries
        with connection.cursor() as cursor:
            # Insert test data into the 'test' table
            cursor.execute("INSERT INTO test (data) VALUES (%s);", ("Hello, World!",))

        # Commit the changes
        connection.commit()

    finally:
        # Close the database connection
        connection.close()


if __name__ == "__main__":
    # Create the 'test' table
    create_table()

    # Insert test data into the 'test' table
    insert_data()
