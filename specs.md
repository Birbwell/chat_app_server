
# CLIENT <=> SERVER PROTOCOLS
> ID_ -> Transmitting Room ID \
> LST -> Transmitting list of rooms \
> MBL -> Transmitting list of members (unimplemented) \

# Encryption
The client/server system uses a hybrid encryption approach. When a new room is created, a symmetric key is then created for that room. When a new user joins that room, the client sends their public key, which is then used to encrypt the symmetric key. The encrypted key is sent back to the client, where it is decrypted and utilized. All messages sent through the server are encrypted with this symmetric key, so all clients who have the key can encrypt and decrypt messages sent.
