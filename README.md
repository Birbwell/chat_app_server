This is the server component of the Chat-App. To use it, simply run the application. To stop the server, press escape.

By default, it will bind to the port 42530.

The way it works:
- Create listener to listen for any incoming connections on port 42530.
- Upon a connection being established, spin up a threat that will listen to any messages sent from via the connection.
- When a message is sent through a connection, copy the contents to all the other threads, and send it to each connection.
- When escape is pressed, safely close all connections/threads and exit.

Essentially, each instance of this program acts as a chat room.
