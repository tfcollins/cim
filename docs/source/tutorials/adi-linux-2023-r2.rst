ADI Linux target: selectable releases
========================================

The self-contained ``adi-linux`` target builds kernel images for pyadi-dt
without pyadi-build. It selects releases and platforms at make time and is kernel-only:
no DTBs, modules installation, root filesystem, BOOT.BIN, flashing, or board
reservation. Boot/hardware validation remains the consumer's responsibility.
This target builds ordinary kernels only; device-tree overlays are maintained
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

   cim init --target adi-linux --source "$PWD" \
     --workspace "$HOME/cim-zynq" --yes
   cd "$HOME/cim-zynq"
   cim makefile
   make sdk-build KERNEL_JOBS=4
   python3 scripts/build-kernel.py --platform zynq \
     --output artifacts/2023_R2/zynq --verify

Defaults are ``KERNEL_RELEASE=2023_R2``, ``KERNEL_PLATFORM=zynq``, and
``KERNEL_JOBS=4``. Select either release (``2023_R2`` or ``2026_R1``) and
platform (``zynq`` or ``zynqmp``) in the same initialized workspace::

   make sdk-build KERNEL_RELEASE=2026_R1 KERNEL_PLATFORM=zynqmp

``KERNEL_OUTPUT`` defaults to ``artifacts/$(KERNEL_RELEASE)/$(KERNEL_PLATFORM)``.
Its references are resolved by make, so overrides cannot accidentally reuse the
other release/platform's default output. An explicit ``KERNEL_OUTPUT=/path``
is an exact directory override: callers must keep custom paths separate.
The four former release/platform-specific target names have been removed;
initialize ``adi-linux`` and select with these variables instead.

The helper and dependency manifest live inside ``targets/adi-linux``. CIM's
``copy_files`` copies the helper into ``scripts/build-kernel.py`` without sibling
paths, including when pinned Git-source initialization extracts only this target.

For remote initialization, pass ``--source https://github.com/tfcollins/cim.git``
and ``--version <reviewed-CIM-commit>`` instead of the local source directory.

Guided build (like HDL)
-----------------------

After ``cim makefile``, use the same entry points as the HDL target::

   make guide-help
   make list-combos
   make guide

The five-step wizard selects release, platform, positive job count and output
folder, then prints shell-quoted build and offline verification commands.
Defaults are ``2023_R2``, ``zynq``, four jobs and
``artifacts/RELEASE/PLATFORM``. Release ``2026_R1`` selects tag
``xlnx_2026.1.0``. Enter accepts a default; ``?`` or ``list`` shows choices;
invalid selections retry. At the final prompt explicitly choose ``build`` or
``verify``; the default is **no**, not an implicit build. ``q``, ``cancel``,
Ctrl-C or EOF exits successfully without starting any further command.

Inspect selections without downloads, verification or builds::

   make guide-dry-run
   python3 scripts/guide-linux.py --dry-run --release 2026_R1 --platform zynqmp --jobs 8

For scripted input use ``--interactive`` (``make guide`` already does this).
Calling the script without an action on noninteractive stdin prints help and
exits. Help, list and dry-run never invoke the builder. Direct execution needs
an explicit action::

   python3 scripts/guide-linux.py --verify --release 2026_R1 --platform zynqmp --output artifacts/2026_R1/zynqmp
   python3 scripts/guide-linux.py --build --release 2023_R2 --platform zynq --jobs 4

The guide passes arguments directly, not through a shell; paths with spaces are
quoted in printed commands and preserved when passed to the helper. However,
upstream kernel make does not support spaces during a fresh build; choose a
space-free output path for builds. Verification of existing artifacts does not
run make. The guide does not save selections or change automated
``make sdk-build`` defaults/overrides. Make variables configure ``sdk-build``;
use wizard prompts or script flags to configure the guide. All guide files are
copied from the same self-contained target during pinned Git-source init.

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
support them. Separate output directories are required for each release/platform pair.

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
       "release": "2023_R2",
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
The consumer minimum checks are schema version, platform, local image path, and
checksum. Run the helper with ``--verify`` for strict pinned provenance validation.

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

   CIM_BIN=/path/to/cim python3 -m unittest discover -s tests -v

With CIM installed, tests generate a real workspace Makefile and verify all four
release/platform overrides, default isolation, job count, and custom output.
Without CIM this integration test is explicitly skipped. Tests also cover both contract variants, build invocation/publication with explicit
fixtures, archive traversal rejection, cache corruption, checksum/provenance
failures, legacy image CRC/header parsing, process locking, and old-generation
retention on failure/rebuild. Where installed, ``mkimage -l`` independently
checks the legacy header. Full pinned cross-builds use the real CLI above;
fixture tests alone are not evidence of a real kernel or a bootable board.
