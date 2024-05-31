# Embedded Bacnet

![an obscure reference to a movie](logo.svg)

"May I never be complete. May I never be content. May I never be perfect."
-- Chuck Palahniuk, Fight Club 

A Rust library to read and write bacnet packets for embedded devices. Bacnet is a protocol used for building automation and control. 
The official spec is unfortunately behind a paywall and so this implementation has been cobbled together by cross referencing multiple code implementations.
The most comprehensive implementation and documentation I have found to be here: https://bacnet.sourceforge.net/ for which I am very grateful. This is also a good resource: http://www.bacnetwiki.com/wiki/

You can use this library to send and receive bacnet packets. However, the entire spec has not been implemented, only the bits I found most important. Use the link above if you want a comprehensive implementation. 

The library requires no standard library or memory allocator so expect to use iterators and loops to when decoding your network packets.

## Getting started

Most of the examples use the `simple` module (see `Bacnet` struct) to perform basic async request-response bacnet queries which should be the most common use case. It is up to you to supply async read and write capabilities by wrapping your favorite network library. Alternatively, you can use this library as a codec in order to have more fine grained control of the communication.

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

## Notes on reference C implementation

The following notes apply to the C repo found here: https://bacnet.sourceforge.net/
The most important part I have found to exist in the `src/bacnet/basic/service` folder which deals with the application part of the stack. 
There is some sense to the acronyms used. 
For example `h_rpm.c` means `handle_read_property_multiple` which is for encoding and decoding read_property_multiple confirmed requests.
Furthermore, `h_rpm_a.c` means `handle_read_property_multiple_acknowledgements` which is for encoding and decoding responses to the request above.

## Unit testing

Unit tests will come when I have more time. Please use the examples for the time being.

## Understanding the internals

At its heart this library is a bacnet codec (encoder / decoder). Because it does not allocate memory AND we have to deal with varying numbers of things (for example a bacnet packet may have any number
of objects in it) the encoding and decoding parts have different representations. 
For example if you wanted to encode a list of objects you would pass a slice from some container because you know, beforehand, how many objects you want to include in the packet. 
When decoding lists of things we use an iterator so the user can collect those object into a vector or simply process them on the fly. 
Internally this is represented as a reader that decodes objects on the fly from a byte buffer.