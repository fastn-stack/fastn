import psycopg2

# PostgreSQL connection parameters
db_params = {
    "host": "localhost",
    "port": 5432,
    "user": "testuser",
    "password": "testpassword",
    "database": "testdb",
}


def fetch_data():
    # Connect to the PostgreSQL database
    connection = psycopg2.connect(**db_params)

    try:
        # Create a cursor object to execute SQL queries
        with connection.cursor() as cursor:
            # Execute a sample query to fetch data from the 'test' table
            print("Executing sample test query")
            cursor.execute("SELECT * FROM test;")

            # Fetch all rows from the result set
            rows = cursor.fetchall()

            print("Showing test data")
            # Print the fetched data
            for row in rows:
                print("ID:", row[0])
                print("Data:", row[1])
                print("-" * 20)

    finally:
        # Close the database connection
        connection.close()


if __name__ == "__main__":
    fetch_data()
