This is a sample to reproduce a race condition leading to a `NotConnected` error
on Windows.

## Principle

This sample connects a `TcpStream` to some public IP (as it happens, one
associated to `yahoo.com`).

It registers it with _writable_ interest, then reregisters with _readable_
interest, then polls. On readable event, it reads.


## Issue on Windows

On Windows, on _readable_ event, calling `read()` results in an error:

```
error: [NotConnected]: A request to send or receive data was disallowed because
the socket is not connected and (when sending on a datagram socket using a
sendto call) no address was supplied. (os error 10057)
```

To reproduce:

    cargo build
    target\debug\mio-not-connected

My results:

```
[1502978550000.697] Attempt 0...
[1502978550000.697] Interest = {writable}
[1502978550000.697] Interest = {readable}
[1502978550000.697] event=Event { kind: Ready {Readable}, token: Token(0) }
error: [NotConnected]: A request to send or receive data was disallowed because
the socket is not connected and (when sending on a datagram socket using a
sendto call) no address was supplied. (os error 10057)
[1502978550000.697] Attempt 0 complete
[1502978550000.697] Attempt 1...
[1502978550000.697] Interest = {writable}
[1502978550000.713] Interest = {readable}
[1502978550000.713] event=Event { kind: Ready {Readable}, token: Token(1) }
error: [NotConnected]: A request to send or receive data was disallowed because
the socket is not connected and (when sending on a datagram socket using a
sendto call) no address was supplied. (os error 10057)
[1502978550000.713] Attempt 1 complete
```

### pollwritable

If we poll before reregistering with _readable_ interest, then the error is
avoided once, but not twice:

    cargo build --features pollwritable
    target\debug\mio-not-connected

My results:

```
[1502978636000.587] Attempt 0...
[1502978636000.587] Interest = {writable}
[1502978636000.681] event=Event { kind: Ready {Writable}, token: Token(0) }
[1502978636000.681] Interest = {readable}
[1502978636000.681] Attempt 0 complete
[1502978636000.681] Attempt 1...
[1502978636000.681] Interest = {writable}
[1502978636000.681] Interest = {readable}
[1502978636000.681] event=Event { kind: Ready {Readable}, token: Token(1) }
error: [NotConnected]: A request to send or receive data was disallowed because
the socket is not connected and (when sending on a datagram socket using a
sendto call) no address was supplied. (os error 10057)
[1502978636000.681] Attempt 1 complete
```

### temporize

If instead, we wait 200ms before reregistering, then it works:

    cargo build --features temporize
    target\debug\mio-not-connected

My results:

```
[1502978686000.277] Attempt 0...
[1502978686000.277] Interest = {writable}
[1502978686000.496] Interest = {readable}
[1502978686000.496] event=Event { kind: Ready {Readable}, token: Token(0) }
spurious event
[1502978686000.496] Attempt 0 complete
[1502978686000.496] Attempt 1...
[1502978686000.496] Interest = {writable}
[1502978686000.715] Interest = {readable}
[1502978686000.715] event=Event { kind: Ready {Readable}, token: Token(1) }
spurious event
[1502978686000.715] Attempt 1 complete
```

There is no problem either if we register with _readable_ interest directly
(without previously requesting _writable_ interest).


## Cause?

These results suggest that there is probably a race condition somewhere.
