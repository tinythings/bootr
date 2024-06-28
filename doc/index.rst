.. Bootr documentation master file, created by
   sphinx-quickstart on Fri Jun 28 10:37:20 2024.
   You can adapt this file completely to your liking, but it should at least
   contain the root `toctree` directive.

Welcome to Bootr's documentation!
=================================

.. toctree::
   :maxdepth: 2

   overview



Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`


.. sidebar:: Links

   * `GitHub Repository <https://github.com/tinythings/bootr>`__
   * `GitHub Issues Tracker <https://github.com/tinythings/bootr/issues>`__

Introduction
------------

**Bootr** is an alternative solution for A/B updates. Instead of using old-fashion image updates,
it is using OCI container means. Similar solutions already exists, but they are bound to certain
constraints, requiring certain 3rd party components, such as OSTree, for example. In contrast,
**bootr** is trying to be as agnostic as possible and run on variety of different Linux distributions,
without explicitly requiring a specific components, those are available only on certain distros.

Another aim where **Bootr** is different is usually A/B approach requires *partitions* on the disk
and space. Same as transactional updates are possible only with specific filesystems, such as Btrfs.
Meanwhile, **Bootr** is trying to run "everywhere" with as minimum as possible requirements.

Use Cases
---------

Main use case for **bootr** is a new-gen Configuration Management approach, where systems are
configured first, "rendered" into an image, and only then are deployed as one-way "omelette":
if an update to the image is required, then the configuration cycle should be repeated. And
this is where OCI infrastructure comes handy: only deltas are sent and applied on the target.


Contributing
------------

Best way to make a progress is to open an issue or submit a Pull request on the GitHub,
or just open an issue with a bug description or a question.
