# Auto Update
This project aims to create a program which listens messages from a server and take action with respect to these receied messages.

## Expected workflow from program
These are the steps that program should perform

1. Perform a dummy task in a loop until a restart event is received from server
2. Upon a restart event
  2.1 Stop the loop
  2.2 Receive update from the server
  2.3 Apply the update
  2.4 Kill itself and start updated version of the program
  2.5 Go back to step 1
