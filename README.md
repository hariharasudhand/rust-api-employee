**To Run the Application**

Go to Source folder and run 

cargo run

then open the terminal 

**To add Employee:**

curl -X POST http://localhost:8000/employees -H "Content-Type: application/json" -d '{"name": "John Doe", "age": 30, "position": "Developer"}'

**To View employees**

curl http://localhost:8000/employees

**To View a particular employee **

curl http://localhost:8000/employees/id



