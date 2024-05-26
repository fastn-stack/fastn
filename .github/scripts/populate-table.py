import sqlite3
import os


def create_table():
    connection = sqlite3.connect(get_database_path(os.environ["FASTN_DB_URL"]))

    try:
        # Create a cursor object to execute SQL queries
        cursor = connection.cursor()
        # Execute a query to create the 'test' table if it doesn't exist
        cursor.execute(
            """
            CREATE TABLE IF NOT EXISTS test (
                id SERIAL PRIMARY KEY,
                data VARCHAR(255) NOT NULL
            );
            """
        )

        cursor.close()

        # Commit the changes
        connection.commit()

    finally:
        # Close the database connection
        connection.close()


def insert_data():
    connection = sqlite3.connect(get_database_path(os.environ["FASTN_DB_URL"]))

    try:
        # Create a cursor object to execute SQL queries
        cursor = connection.cursor()
        # Insert test data into the 'test' table
        cursor.execute("INSERT INTO test (data) VALUES ('Hello, World!');")

        # Commit the changes
        connection.commit()

    finally:
        # Close the database connection
        connection.close()


# Function to strip 'sqlite:///' prefix if present
def get_database_path(uri):
    prefix = 'sqlite:///'
    if uri.startswith(prefix):
        return uri[len(prefix):]
    return uri


if __name__ == "__main__":
    # Create the 'test' table
    create_table()

    # Insert test data into the 'test' table
    insert_data()
