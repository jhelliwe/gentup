A program written in Rust, intended to be used on Gentoo Linux, which takes care of running a full update on this rolling Linux distribution

Compile with "cargo build --release"

This updater takes care of much of the heavy lifting that a Gentoo administrator has to do in order to keep their
Gentoo installation up to date.

Features:
- This updater depends on eix, eclean-kernel, gentoolkit and elogv, so if these are not installed, the updater will install them.
- The updater supports two configuration files, and these can be managed with "gentup --setup". These control if the
  updater will perform a disk-space cleanup by default, a post-update filesystem trim by default, and enables the user to
  configure an email address to send notification emails to (This feature depends on the user setting up their sendmail environment
  separately.) The second configuration file contains a list of packages to install by default if they are missing.
- The updater optionally installs the set of commonly installed packages, useful for a brand new Gentoo install.
  The list of packages is editable in the --setup mode.
- The updater will check to see if the last "emerge --sync" was too recent to avoid syncing too often
- The updater lists any packages due an upgrade, and optionally pre-fetches the package sources
- The updater emails a list of Gentoo news articles to the user, if any are found
- The updater will then update all packages on the system
- The updater will merge in any confguration file changes due to package upgrades
- After the update, a list of package install elogs is displayed
- The updater lists and cleans orphaned dependencies
- The updater lists and repairs any broken reverse dependencies
- The updater checks the sanity of the /etc/portage configuration files
- The updater optionally removes old unused source distribution tarballs
- The updater optionally cleans up old kernels from /boot, /lib/modules and the GRUB configuration files
- The updater then optionally performs an fstrim of all filesystems
