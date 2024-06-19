# Bootr Overview

Bootr is using OCI layers to update the entire system.

## Directory Structure

Root directory contains `/bootr` direcory, which has all the required
data to update the rest of the system. Bootr _does not_ use `/etc`
directory which might be even read-only, after all.

`/bootr/bootr.conf`: main configuration of Bootr.

## Configuration File

```yaml
# Automated system update
update:
  # Perform auto-update or not
  auto: true | false
  
  # Type of update check: polling or event-based (in a distant future)
  type: poll | event
  
  # Polls every x minutes or hours
  check: 0m|0h

# Round-robin versions (default a/b)
versions:
  - a
  - b
```

## `/bootr/system/`

This is the updates storage directory. It contains the updates,
delivered by OCI means and are used for the next boot. The
directory contains subdirectories such as `/a`, `/b` etc (can be
configured on demand) to have variants of the system.

Each of those subdirectory has own tracking of OCI layers in the
following format, e.g. `/a`:

```
/bootr/system/a
              |
              +-- layers/
			  +-- rootfs/
			  +-- status.yaml
```

The `layers` is a directory, which contains empty files with named
after layer files (SHA checksum), so Bootr will not re-download and
process them again.

The `rootfs` is a directory, which contains updated rootfs.

The `status.yaml` is a YAML file, which has `key: value` format.
It contains the information about the rootfs and its status, such as:
- when it was last updated
- is it currently running rootfs or not
- last checksum
- which OCI container, vendor, author, packager, publisher etc

