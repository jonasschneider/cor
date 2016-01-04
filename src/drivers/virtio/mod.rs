// We can now talk to the actual virtio device
// via the CPU's I/O pins directly. A couple of helpful references:
//
// http://ozlabs.org/~rusty/virtio-spec/virtio-0.9.5.pdf
//     This is the actual virtio spec.
//
// http://ozlabs.org/~rusty/virtio-spec/virtio-paper.pdf
//     This is an academic paper describing the virtio design and architecture,
//     and how a virtqueue works and is implemented.
//
// https://www.freebsd.org/cgi/man.cgi?query=virtio&sektion=4
//     This is actually a FreeBSD manpage that gives a pretty good high-
//     level overview of how the guest kernel usually interacts with the
//     virtio interfaces and how it presents them to the guest OS's file
//     system.

pub mod serial;
pub mod block;

mod virtq;
mod vring;
mod pci;
