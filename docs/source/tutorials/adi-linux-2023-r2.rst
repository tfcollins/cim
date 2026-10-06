ADI Linux target: selectable releases
================================================================================

The self-contained ``adi-linux`` target builds kernel images for pyadi-dt
without pyadi-build. It selects releases and platforms at make time and is kernel-only:
no DTBs, modules installation, root filesystem, BOOT.BIN, flashing, or board
reservation. Boot/hardware validation remains the consumer's responsibility.
This target builds ordinary kernels only; device-tree overlays are maintained
separately and are not generated or modified here. Upstream defconfigs and their
complete embedded radio firmware selections are preserved without pruning.

Requirements
--------------------------------------------------------------------------------

Use a Linux x86_64 host, Python 3.11.8+ (including tarfile's data extraction
filter), GNU make, GCC for host utilities, bc, bison, flex, libssl-dev and
libelf-dev. Ubuntu 24.04 is the supported dependency recipe. Allow several GB
of working space and an initial network download of source and toolchains.
No root privileges are required after host dependencies are installed.

Build with CIM
--------------------------------------------------------------------------------

Initialize from an immutable CIM manifest commit containing the unified target
and guide (the pin below is a concrete known revision, not a moving branch)::

   CIM_MANIFEST_COMMIT=6e3728f12172709274e2393af062e2c4889345ea
   cim init --target adi-linux --source https://github.com/tfcollins/cim.git \
     --version "$CIM_MANIFEST_COMMIT" --workspace "$HOME/cim-zynq" --yes
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

The CIM manifest commit pins the helper, guide and target definition; the
helper separately pins the Linux source and toolchain archives. For local
development only, replace the remote source/version arguments with
``--source /absolute/path/to/cim-checkout``. Install CIM separately and, if
host dependencies are missing, run ``cim install os-deps --yes`` in the
workspace (this may require sudo). Initialization alone does not compile Linux.

Supported combinations and safe previews
--------------------------------------------------------------------------------

Every pair uses four jobs by default. Output paths below are relative to the
initialized workspace; the manifest is ``artifacts.json`` inside each output.
The final image is in a generation subdirectory, not directly at the root.

.. list-table::
   :header-rows: 1

   * - KERNEL_RELEASE
     - KERNEL_PLATFORM
     - Default KERNEL_OUTPUT
     - Image
   * - 2023_R2
     - zynq
     - artifacts/2023_R2/zynq
     - uImage
   * - 2023_R2
     - zynqmp
     - artifacts/2023_R2/zynqmp
     - Image
   * - 2026_R1
     - zynq
     - artifacts/2026_R1/zynq
     - uImage
   * - 2026_R1
     - zynqmp
     - artifacts/2026_R1/zynqmp
     - Image

These executable documentation examples preview all supported selections,
without downloads or artifact changes:

.. code-block:: bash
   :name: linux-offline-examples

   python3 scripts/guide-linux.py --list
   python3 scripts/guide-linux.py --dry-run --release 2023_R2 --platform zynq --jobs 4
   python3 scripts/guide-linux.py --dry-run --release 2023_R2 --platform zynqmp --jobs 4
   python3 scripts/guide-linux.py --dry-run --release 2026_R1 --platform zynq --jobs 4
   python3 scripts/guide-linux.py --dry-run --release 2026_R1 --platform zynqmp --jobs 4

Guided build (like HDL)
--------------------------------------------------------------------------------

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
``quit`` also cancels. Numbered release/platform selections are accepted in
listed order. Cancellation does not roll back work already started.

Inspect selections without downloads, verification or builds::

   make guide-dry-run
   python3 scripts/guide-linux.py --dry-run --release 2026_R1 --platform zynqmp --jobs 8

For scripted input use ``--interactive`` (``make guide`` already does this).
Calling the script without an action on noninteractive stdin prints help and
exits. Help, list and dry-run never invoke the builder. Direct execution needs
an explicit action::

   python3 scripts/guide-linux.py --verify --release 2026_R1 --platform zynqmp --output artifacts/2026_R1/zynqmp
   python3 scripts/guide-linux.py --build --release 2023_R2 --platform zynq --jobs 4

Guide options are ``-i/--interactive``, ``-l/--list``, ``--release``,
``--platform``, ``-j/--jobs``, ``--output``, and ``-h/--help``. At most one of
``--dry-run``, ``--build`` or ``--verify`` may be supplied. Combining an action
with ``--interactive`` still prompts; dry-run stops before confirmation, while
other interactive actions ask whether to build, verify or cancel. The guide
does not expose the helper's ``--cache`` or ``--force`` options.

The guide passes arguments directly, not through a shell; paths with spaces are
quoted in printed commands and preserved when passed to the helper. However,
upstream kernel make does not support spaces during a fresh build; choose a
space-free output path for builds. Verification of existing artifacts does not
run make. The guide does not save selections or change automated
``make sdk-build`` defaults/overrides. Make variables configure ``sdk-build``;
use wizard prompts or script flags to configure the guide. All guide files are
copied from the same self-contained target during pinned Git-source init.

Reference and consumer handoff
--------------------------------------------------------------------------------

See :doc:`/reference/adi-linux` for the standalone helper options, exact
2023_R2 pins, image packaging, artifact schema, checksums, provenance and
cache/concurrency guarantees. See :doc:`adi-linux-2026-r1` for the exact
2026_R1 source and :doc:`/howto/adi-linux-pyadi-dt` for consumer release mapping.

Verification
--------------------------------------------------------------------------------

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
