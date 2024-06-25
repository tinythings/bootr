# Bootr

**Bootr** ("booter") belongs to niche of bootable containers.

This is an alternative implementation of bootable containers, very
similar to [bootc](https://containers.github.io/bootc/), but doesn't
require OSTree, RPMs and `systemd`, is not bound solely to Fedora/RedHat and
is designed to be as agnostic as possible.

Main goal and focus is on system updates, using OCI container
mechanisms, being a part of next-generation configuration management
system, relying on containers and Kubernetes.

*Bootr* is intended to be used as a new way of configuration management,
where configuration is doing "home office" being "remote" to the system,
so the actual system is only updated from already tested OCI image published
to the official registry. This removes [configuration drift](https://www.puppet.com/blog/configuration-drift#:~:text=Back%20to%20top-,What%20is%20Configuration%20Drift%3F,on%20system%20performance%20and%20security.) 
wher the system changes overtime towards unexpected direction, and prevents "reliable failures"
as typically done with transactional updates, where system is reliably updated
with mistakes, brought by broken packaging. 😉

WARNING: This project is in extremely early stage and should be considered
as a proof of concept at its best.
