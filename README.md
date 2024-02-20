A program written in Rust, intended to be used on Gentoo Linux, which takes care of running a full update on this rolling Linux distribution

Build time dependencies: CrossTerm, Terminal Spinners, FileTime, Chrono and Execute

Compile with "cargo build --release"

This updater takes care of much of the heavy lifting that a Gentoo administrator has to do in order to keep their
Gentoo installation up to date.

Features:
- This updater depends on eix, eclean-kernel, gentoolkit and elogv, so if these are not installed, the updater will install them.
- Next, the updater installs a set of commonly installed packages, useful for a brand new Gentoo install. This step is only
  performed the first time the updater is ever run. The list of packages is editable in the file /etc/default/gentup
- The updater will check to see if the last "emerge --sync" was too recent to avoid syncing too often
- The updater lists any packages due an upgrade, and optionally pre-fetches the package sources
- The updater presents a list of Gentoo news articles to the user
- The updater will then update all packages on the system
- The updater will merge in any confguration file changes due to package upgrades
- After the update, a list of ELOGs is displayed
- The updater lists and cleans orphaned dependencies
- The updater lists and repairs any broken reverse dependencies
- The updater checks the sanity of the /etc/portage configuration files
- The updater optionally removes old unused source distribution tarballs
- The updater optionally cleans up old kernels from /boot, /lib/modules and the GRUB configuration files
- The updater then optionally performs an fstrim if the root filesystem resides on non-rotational storage
