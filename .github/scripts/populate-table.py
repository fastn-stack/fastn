import psycopg2
import os

def create_table():
    connection = psycopg2.connect(os.environ["FASTN_DB_URL"])

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
    connection = psycopg2.connect(os.environ["FASTN_DB_URL"])

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
