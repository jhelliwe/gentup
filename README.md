A short Rust program intended to be used on Gentoo Linux, which takes care of running an O/S update.
I only wrote this as a learning exercise because I hadn't ever used Rust before.
It may come in useful?

Compile with "cargo build --release"

Updating a Gentoo system can be a lot of work. The user has to make sure they
don't abuse the sync rotation of rsync.gentoo.org by syncing too often;
They have to take care of making sure that they update the portage package first
before performing a world update. If an update to gcc is available, it makes more
sense to update gcc first too. Then there are the ELOG readme's to read, orphaned
dependencu removal, revdep checks, making sure that your /etc/portage/* files are
sane, cleaning up your distfiles, and other cleanups like cleaning up old unused
kernels from /boot and /lib/modules. All these are manual steps. This program will
guide the Gentoo admin through the tasks. It's useful to me, maybe it could be
useful for other people.
