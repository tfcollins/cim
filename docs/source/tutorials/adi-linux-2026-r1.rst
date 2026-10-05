ADI Linux target: 2026_R1 selection
========================================

The self-contained ``adi-linux`` kernel-only target supports
``KERNEL_RELEASE=2026_R1`` with either platform. It retains the :doc:`adi-linux-2023-r2` prerequisites,
image packaging and schema-version-1 artifact interface. No overlays, BOOT.BIN,
root filesystem, module installation, flashing or hardware testing are included.

Exact source and compatibility
------------------------------

The release selector ``2026_R1`` maps to the **tag** ``xlnx_2026.1.0`` in
``analogdevicesinc/linux``, not a branch or a similarly named ADI release ref:

* Commit: ``b47bbbe8ca7bc582c96251fa30d86e55de363f68``.
* Commit tar.gz SHA-256:
  ``0886274c27356de24e3b7030817e1669d1e14b42a17a2c24055d74dad16472ed``.
* Inspected kernel version: 6.12.77. Its ``scripts/min-tool-version.sh`` requires
  GCC >= 5.1.0 and binutils >= 2.25.0 for these architectures. Both targets use
  the existing checksum-pinned kernel.org GCC 12.2.0 cross-toolchain archives.
* The archive contains ``zynq_xcomm_adv7511_defconfig`` for ARM and
  ``adi_zynqmp_defconfig`` for ARM64. Their complete ``CONFIG_EXTRA_FIRMWARE``
  selections (24 and 38 files respectively) are present under ``firmware/``.
  They include Navassa calibration/profile files and, for ARM64, ADRV9025
  firmware/profiles. Nothing is pruned or downloaded from an unpinned firmware
  repository. ``CONFIG_EXTRA_FIRMWARE_DIR="./firmware"`` is preserved.

Build and validate
------------------

From a checkout of this repository, build both platforms in one workspace::

   repo="$PWD"
   workspace="$HOME/cim-linux"
   cim init --target adi-linux --source "$repo" --workspace "$workspace" --yes
   (cd "$workspace" && cim makefile)
   for platform in zynq zynqmp; do
     (cd "$workspace" && make sdk-build KERNEL_RELEASE=2026_R1 \
       KERNEL_PLATFORM="$platform" KERNEL_JOBS=4)
     python3 "$workspace/scripts/build-kernel.py" --release 2026_R1 \
       --platform "$platform" --output "$workspace/artifacts/2026_R1/$platform" --verify
   done

For pinned remote initialization, replace ``--source "$repo"`` with
``--source https://github.com/tfcollins/cim.git --version <reviewed-CIM-commit>``.
The target contains its own helper; Git initialization need not retrieve
sibling directories. ``KERNEL_OUTPUT`` overrides the release/platform-specific output.

The standalone CLI is also supported::

   python3 targets/adi-linux/build-kernel.py --release 2026_R1 \
     --platform zynq --output /absolute/output/2026_R1/zynq --jobs 4
   python3 targets/adi-linux/build-kernel.py --release 2026_R1 \
     --platform zynqmp --output /absolute/output/2026_R1/zynqmp --jobs 4

Omitting ``--release`` (or ``KERNEL_RELEASE`` in make) selects ``2023_R2``.
The unified target defaults to platform ``zynq``. Pass the same release to
``--verify``. Stdout remains only the absolute ``artifacts.json`` path; logs
are on stderr. The manifest still has ``schema_version: 1`` and the same
platform, image-path and checksum fields. Provenance additionally records
``release``, the exact source ref/type, archive pin and builder checksum.
Different releases cannot reuse one another's output: use separate directories,
or explicitly rebuild with ``--force``. Updated helpers intentionally reject
old builder-provenance caches; previously published images remain untouched.

Offline tests and real cross-builds are distinct verification tiers. Successful
image validation is not a boot or HIL qualification.
