from flask import Flask, jsonify
import psycopg2
import os

app = Flask(__name__)


def fetch_data():
    # Connect to the PostgreSQL database
    connection = psycopg2.connect(os.environ["FASTN_DB_URL"])

    try:
        # Create a cursor object to execute SQL queries
        with connection.cursor() as cursor:
            # Execute a query to fetch data from the 'test' table
            cursor.execute("SELECT * FROM test;")

            # Fetch first row from the result set
            row = cursor.fetchone()

            data = dict()
            if row is not None:
                data = {
                    "id": row[0],
                    "data": row[1],
                }

    finally:
        # Close the database connection
        connection.close()

    return data


@app.route('/get-data/', methods=['GET'])
def get_data():
    # Fetch data from the 'test' table
    data = fetch_data()

    # Return the data as JSON
    json_result = jsonify(data)
    print(json_result)
    return json_result


if __name__ == '__main__':
    # Run the Flask application on port 5000
    print("Starting python server")
    app.run(port=5000)
