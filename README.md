# Auto Update
This project aims to create a program which listens messages from a server and takes actions with respect to these receied messages.

## Expected workflow from program
These are the steps that program should perform

1. Perform a dummy task in a loop until a restart event is received from server
2. Upon a restart event
3. Stop the loop
4. Receive update from the server
5. Apply the update
6. Kill itself and start updated version of the program
7. Go back to step 1
