..overview

Overview
========

The **bootr** is aiming to implement transactional, in-place operating system updates via OCI container images.

To prevent configuration drifts and failures after in-place updates, a typical solution is to use "A/B"
updates via images. However, performing this remotely is a big challenge, especially calculating the "delta" image,
which sometimes might be even bigger than the original one.

This is where OCI container model comes to the rescue. A typical OCI container model is using "layers" and it
was very successful. The **bootr** project is using the same approach for bootable host systems, using just
OCI containers as a transport and delivery format for base operating system updates.

Status
------

Right now, this project cannot be even called "Beta" and more like a "Proof of Concept". However, it is not
meant to be a thrown away prototype and supposed to grow into a stable software.


Alternative Projects
--------------------

Another project, based on OSTree and aiming very similar goals is * `project bootc <https://github.com/containers/bootc>`__ *
