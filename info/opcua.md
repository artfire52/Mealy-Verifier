# Vocabulary for OPC UA inference

## Input vocabulary

- hello: Hello message of OPC UA
- open\_secure\_channel\_request: request to open a secure channel
- open\_secure\_channel\_request\_wrong: request to open a secure channel using an unauthorized certificate
- get\_endpoint\_request: get endpoint request to discover allowed policy (cryptographic suites) on a server.
- close\_secure\_channel: request to close a secure channel
- create\_session: create a session
- active\_session: request to activate a session using username and password.
- active\_session\_false\_token\_id: request to activate a session using username and password with an incorrect token id field.
- active\_session\_anon: request to activate an anonymous session 
- active\_session\_wrong\_user: request to activate a session using unauthorized username and password
- active\_session\_cert: request to activate a session with a certificate
- active\_session\_cert\_wrong: request to activate a session with an unauthorized certificate
- read\_req: request to read in the address space
- write\_req: request to write in the address space
- nullsize: request to open secure channel with a null size field
- open\_secure\_channel\_c\_chunk: request to open a secure channel with an error in the field chunk

## Output vocabulary

- Service\_fault: error message related to service
- Err: error messages different than service fault
- Ack: respond to an hello message
- OpnRepOK: response to a successful opening of a secure channel
- OpnRepNOK: response to a failed opening of a secure channel
- GepResNOK: successful response to a get endpoint request
- GepResOK: response to a failed get endpoint request
- Clo: closure of a secure channel
- CreSesResOK: successful creation of a session
- CreSesResNOK: failure to create of a session
- CloSesResOK: successful closure of a session
- CloSesResNOK: failed closure of a session
- AcSesResOK: successful activation of a session
- AcSesResNOK: failed activation of a session
- ReadRepOK: successful reading in the address space
- ReadRepNOK: failed reading in the address space
- WriteRepOK: successful writing in the address space
- WriteRepNOK: failed writing in the address space
- ParseErr: the mapper failed to parse the message
- Eof: End of the connection
- No resp: server does not respond


