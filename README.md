# Embedded Bacnet

![an obscure reference to a movie](logo.svg)

"May I never be complete. May I never be content. May I never be perfect."
-- Chuck Palahniuk, Fight Club 

A Rust library to read and write bacnet packets for embedded devices. Bacnet is a protocol used for building automation and control. 
The official spec is unfortunately behind a paywall and so this implementation has been cobbled together by cross referencing multiple code implementations.
The most comprehensive implementation and documentation I have found to be here: https://bacnet.sourceforge.net/ for which I am very grateful. This is also a good resource: http://www.bacnetwiki.com/wiki/

You can use this library to send and receive bacnet packets. However, the entire spec has not been implemented, only the bits I found most important. Use the link above if you want a comprehensive implementation. 

The library requires no standard library or memory allocator so expect to use iterators and loops to when decoding your network packets.

## Current state

This software is currently in alpha state so you can expect the API to change significantly in the near future. It will be stabilized once it has been used for a real-world use case. 
This library is being developed in tandem with such a real-world use case so its stability is dependent on it.

## How it works

Bacnet is a protocol that can work on top of many transport protocols. This implementation only works with Bacnet IP which uses UDP packets. Like many protocols this one has layers. 
An application layer wrapped in a network layer wrapped in a data link layer wrapped in a udp packet. Like this:

```
UdpPacket
|
-> DataLink (about the connection)
   |
   -> NetworkPdu (flags and the reason for the message)
      |
      -> ApplicationPdu (the payload)
```

Where Pdu stands for protocol data unit.

This is what a typical bacnet client would do with this library: Send an old-school broadcast UDP packet out to the standard bacnet port and listen for replies on the same port. 
This is done using an unconfirmed `who_is` pdu (protocol data unit). When a controller is found by decoding the inevitable `i_am` unconfirmed response the client can then send udp packets directly to that controller.
A typical request would be a confirmed `property_read` pdu to get a list of objects from the controller. Confirmed requests are tagged with an identifier so the controller can respond to the exact request sent.

## Why build this

I was frustrated by all the acronyms and assumed know-how and wanted to make something that a beginner would find easier to use. 
For example, I will tend to use verbose file names like `read_property.rs` instead of `rp.c`. I assume that the user will use a language server like `rust-analyzer` with autocomplete.
The existing rust implementations seemed to be abandoned and modern Rust capabilities offer new modelling options so this library takes a fairly different approach.

## Design philosophy

I wanted to make a library that was easy to navigate.
For that reason I chose not to abstract things behind traits because it's really just unnecessary most of the time and I really despise navigation black holes.
The code layout should be as obvious as possible and you shouldn't have to read the entire codebase to find what you want to do. 
You may notice quite a bit of code duplication but this is temporary. I like to get a feel for the extent and distribution of duplication before I conpact things with a refactor.
Same thing with error handling; ideally this crate would not panic in response to invalid input data but this will take some time to get right.
Unfortunately, the more I learn about Bacnet, the less capable I'll be at seeing this thing from the eyes of a beginner. Noob questions more than welcome.

## Notes on reference C implementation

The following notes apply to the C repo found here: https://bacnet.sourceforge.net/
The most important part I have found to exist in the `src/bacnet/basic/service` folder which deals with the application part of the stack. 
There is some sense to the acronyms used. 
For example `h_rpm.c` means `handle_read_property_multiple` which is for encoding and decoding read_property_multiple confirmed requests.
Furthermore, `h_rpm_a.c` means `handle_read_property_multiple_acknowledgements` which is for encoding and decoding responses to the request above.

## Unit testing

You may notice that there are no (or very little) unit tests. This is because I am not (yet) convinced that the parsing is correct and I don't want to write tests to cover incorrect logic.
Once I have tested this with multiple real-world controllers I will be happier with what tag numbers are unchanging and which ones are implementation specific. The tests will come.