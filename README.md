# voxa-server

Self host your voxa community server with customizable plugins

# Protocol

The protocol involves mixing simple principles with complex principles.

1. If the client is online then the client will connect to all the servers, not just the "current server".
2. If a server sends a `Message` the client processes it as a notification.
3. A `Message` should only be sent to the client if the server knows that the user that the client is wrapping has been on this server before the messages were present, otherwise it will send a `InitialMessage`.

## Handshake

Client -> Connect -> Server

Server -> Public RSA -> Client

Client -> RSA encrypted AES key -> Server

Client -> (AES) `{ session_id: <Session-Id> }` -> Server

Server -> Validate Session-Id via HTTP -> Voxa cloud

Server -> (AES) `{ id: 0, result: "ok", session_id: <Server-Session> }` -> Client

## Sending a message

Client -> (AES) `{ id: 10, SendMessage: { content: "This is a test", channel: <Channel-Id> } }` -> Server

Server -> (AES) `{ id: 10, result: "ok" }` -> Client

## Receiving a message

Server -> (AES) `{ id: 23, Message: { content: "This is a test", author: <User-Id>, channel: <Channel-Id> } }` -> Client

Client -> (AES) `{ id: 23, result: "ok" }` -> Server

## Sending and receiving voice chat

Client -> (AES) `{ ConnectVoice: { channel: <Channel-Id> } }` -> Server

Client -> (AES) `{ VoiceData: [ <Voice-Data> ] }` -> Server

Server -> (AES) `{ <User-Id>: [ <Voice-Data> ] }` -> Client

Client -> (AES) `{ DisconnectVoice: [] }` -> Server

## Edge cases

- **Client closes TCP prematurely on voice chat**: Server should handle the voice disconnecting on it's own.
- **Server disconnects Client from voice chat**: In the case that a Server needs to disconnect a Client, the server will send the `DisconnectVoice` method to the Client to notify that any further `VoiceData` won't be processed.
- **Invalid or outdated protocol**: Both Server and Client should check for a update and update if `auto_update` is enabled, it's important to keep the server always updated.
- **`Message` cannot be sent at the moment**: The server will sent after the next [Handshake](#handshake)