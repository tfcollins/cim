Maintained ADI Linux 2023_R2 targets
========================================

Targets ``adi-linux-2023-r2-zynq`` and ``adi-linux-2023-r2-zynqmp`` build
kernel images for pyadi-dt without pyadi-build. These are kernel-only targets:
no DTBs, modules installation, root filesystem, BOOT.BIN, flashing, or board
reservation. Boot/hardware validation remains the consumer's responsibility.
These targets build ordinary kernels only; device-tree overlays are maintained
separately and are not generated or modified here. Upstream defconfigs and their
complete embedded radio firmware selections are preserved without pruning.

Requirements
------------

Use a Linux x86_64 host, Python 3.11.8+ (including tarfile's data extraction
filter), GNU make, GCC for host utilities, bc, bison, flex, libssl-dev and
libelf-dev. Ubuntu 24.04 is the supported dependency recipe. Allow several GB
of working space and an initial network download of source and toolchains.
No root privileges are required after host dependencies are installed.

Build with CIM
--------------

From a checkout of this CIM repository::

   cim init --target adi-linux-2023-r2-zynq --source "$PWD" \
     --workspace "$HOME/cim-zynq" --yes
   cd "$HOME/cim-zynq"
   cim makefile
   make sdk-build KERNEL_JOBS=4
   python3 scripts/build-kernel.py --platform zynq \
     --output artifacts/zynq --verify

Use ``adi-linux-2023-r2-zynqmp`` for ZynqMP; its default output directory is
``artifacts/zynqmp``. ``KERNEL_OUTPUT`` selects another output directory.
The helper is copied locally via CIM's existing ``copy_files`` support; no
Rust changes or new CIM CLI commands are required.

Stable standalone helper CLI
----------------------------

The same implementation is callable directly from a pinned CIM checkout::

   python3 targets/adi-linux/build-kernel.py --platform zynq \
     --output /absolute/output/zynq --jobs 4
   python3 targets/adi-linux/build-kernel.py --platform zynqmp \
     --output /absolute/output/zynqmp --jobs 4

Exit zero means the manifest and image have been validated. Stdout contains
only the absolute ``artifacts.json`` path; command/build logs go to stderr.
``--verify`` validates offline without downloads or builds. ``--cache DIR``
selects the archive cache (default ``~/.cache/cim/adi-linux``). ``--force``
rebuilds without invalidating a previously published generation on failure.
Do not use output paths containing spaces: upstream kernel make does not
support them. Separate output directories are required for each platform.

Pins and packaging
------------------

* Source: ``analogdevicesinc/linux`` **branch** ``2023_R2``, frozen at commit
  ``86d61468a7856e952c7ca237f798d86d6abd2e27``. The uppercase name is not a tag;
  the distinct lowercase ``2023_r2`` tag is intentionally not substituted.
  The commit archive is SHA-256 pinned in the helper.
* Compiler: kernel.org x86_64 GCC 12.2.0 nolibc cross-toolchains, with SHA-256
  values from the vendor's ``12.2.0/sha256sums.asc``. Both archives are pinned;
  host cross-compilers and mutable extracted caches are never used.
* Zynq: ``ARCH=arm``, ``zynq_xcomm_adv7511_defconfig``, ``zImage``, wrapped as
  a legacy Linux/ARM/kernel/uncompressed ``uImage`` with **both load and entry
  0x8000**. The stdlib packager emits the U-Boot 64-byte header with header
  and payload CRC32 and timestamp zero; ``mkimage`` is not a build dependency.
* ZynqMP: ``ARCH=arm64``, ``adi_zynqmp_defconfig``, raw ``Image``.

The compiler prefixes are ``arm-linux-gnueabi-`` and ``aarch64-linux-``.
They deliberately differ from pyadi-build's distro prefixes
``arm-linux-gnueabihf-`` / ``aarch64-linux-gnu-``: the kernel does not link
userspace libc or use userspace hard-float ABI. These are kernel-only
compilers, not a userspace SDK.

Every actual build extracts verified source/compiler archives into a private
staging directory and uses ``make O=<separate-build-directory>``. Build identity
and timestamp variables are fixed, but bit-for-bit reproducibility across
arbitrary host utility versions is not claimed.

Artifact contract v1
--------------------

``OUTPUT/artifacts.json`` is the only discovery interface. Consumers must not
search stale build directories or guess image names. Required top-level fields::

   {
     "schema_version": 1,
     "platform": "zynq",
     "kernel_image": "/absolute/output/zynq/image-<generation>/uImage",
     "sha256": "<64 lowercase hexadecimal characters>",
     "provenance": {
       "source": {
         "url": "<commit archive URL>",
         "sha256": "<archive checksum>",
         "commit": "86d61468a7856e952c7ca237f798d86d6abd2e27",
         "ref": "2023_R2"
       },
       "toolchain": {
         "url": "<pinned archive URL>",
         "sha256": "<archive checksum>",
         "version": "12.2.0",
         "cross_compile": "arm-linux-gnueabi-"
       },
       "arch": "arm",
       "defconfig": "zynq_xcomm_adv7511_defconfig",
       "packaging": {
         "format": "uImage",
         "load_address": 32768,
         "entry_address": 32768
       },
       "builder_sha256": "<helper file checksum>"
     }
   }

For ZynqMP, platform is ``zynqmp``, basename/format is ``Image``, arch is
``arm64``, defconfig is ``adi_zynqmp_defconfig``, cross_compile is
``aarch64-linux-``, and both addresses are JSON ``null``. SHA-256 always covers
the final packaged image, not the raw Zynq payload. Absolute image paths are
local to the build host: consumers transferring artifacts must copy the image
and explicitly establish their own local path, rather than treating a remote
manifest as locally usable.

The pyadi-dt integration uses ``ADIDT_KERNEL_ARTIFACTS_ZYNQ`` and
``ADIDT_KERNEL_ARTIFACTS_ZYNQMP`` to point to the corresponding manifest.
It must check schema version, platform, checksum, and provenance before use.

Cache and concurrency
---------------------

Archive files are content-addressed and rehashed on every reuse. A corrupt
cache entry fails closed: remove the named corrupt archive explicitly to
retry. Downloads are atomically renamed only after checksum verification;
archives use Python's safe data extraction filter. Extracted trees are never
cached. Use a private local cache/output directory, not one writable by
untrusted users (checksums are integrity checks, not authentication).

A per-output POSIX flock serializes builds and a per-archive flock serializes
downloads. Completed images get new generation directories; a single atomic
rename publishes the manifest only after image validation. Old generations
remain valid for existing readers, including during forced rebuilds. There is
no automatic garbage collection: remove unused generations only when no
consumer references them. A terminated process can leave hidden staging files
or an unreferenced generation, but never publishes partial output. Atomic
publication here covers process failure/concurrent readers, not power-loss
filesystem durability. Do not share these locks on filesystems without reliable
POSIX flock/rename semantics.

Verification
------------

Fast tests (no network)::

   python3 -m unittest discover -s tests -v

Tests cover both contract variants, build invocation/publication with explicit
fixtures, archive traversal rejection, cache corruption, checksum/provenance
failures, legacy image CRC/header parsing, process locking, and old-generation
retention on failure/rebuild. Where installed, ``mkimage -l`` independently
checks the legacy header. Full pinned cross-builds use the real CLI above;
fixture tests alone are not evidence of a real kernel or a bootable board.
