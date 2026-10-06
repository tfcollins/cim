Use ADI Linux artifacts with pyadi-dt
================================================================================

Build before reserving hardware
--------------------------------------------------------------------------------

Initialize the immutable ``adi-linux`` workspace in
:doc:`/tutorials/adi-linux-2023-r2` first. In that workspace, this automated
recipe builds and strictly verifies both platforms for 2026_R1; omit a platform
if your tests do not need it. It requires the host dependencies and network
access for uncached archives, but no board:

.. code-block:: bash

   set -e
   for platform in zynq zynqmp; do
     make sdk-build KERNEL_RELEASE=2026_R1 KERNEL_PLATFORM="$platform" KERNEL_JOBS=4
     python3 scripts/build-kernel.py --release 2026_R1 --platform "$platform" \
       --output "artifacts/2026_R1/$platform" --verify
   done

Use ``KERNEL_JOBS`` rather than outer ``make -j`` to control kernel parallelism.
For 2023_R2 change both the build selector and verification release/output.
Never verify a 2026_R1 image with the helper's implicit 2023_R2 default.
The helper prints only the absolute manifest path to stdout; the guide prints
human-readable summaries, and make may print recipes. Do not parse guide/make
stdout as if it were the helper's machine-readable output.

Release mapping and manifest handoff
--------------------------------------------------------------------------------

The pyadi-dt consumer integration uses a different public spelling for 2026:

.. list-table::
   :header-rows: 1

   * - pyadi-dt ADIDT_CIM_RELEASE
     - CIM KERNEL_RELEASE / helper --release
     - Linux source ref
   * - 2023_R2 (default)
     - 2023_R2
     - branch 2023_R2, frozen commit
   * - 2026-R1
     - 2026_R1
     - tag xlnx_2026.1.0, frozen commit

Do not pass ``2026-R1`` to the CIM helper or ``2026_R1`` to the consumer's
public selector. The exact source commit/archive pin is documented in
:doc:`/tutorials/adi-linux-2026-r1`.

After the preceding verification succeeds, export paths from the same workspace
for a pyadi-dt checkout that includes the CIM consumer integration:

.. code-block:: bash

   export ADIDT_CIM_RELEASE=2026-R1
   export ADIDT_KERNEL_ARTIFACTS_ZYNQ="$PWD/artifacts/2026_R1/zynq/artifacts.json"
   export ADIDT_KERNEL_ARTIFACTS_ZYNQMP="$PWD/artifacts/2026_R1/zynqmp/artifacts.json"

Export only the platform(s) actually built. The consumer reads the manifest,
not a guessed ``uImage``/``Image`` path. It checks schema, platform, local image,
checksum and selected source provenance; a wrong release fails rather than
silently falling back or rebuilding. CIM's strict ``--verify`` additionally
compares the full expected provenance, including the helper checksum.

Alternatively, run pyadi-dt's ``.github/scripts/prepare_cim_kernel.py`` from its
checkout, with an installed CIM executable and reviewed source/full commit.
Its ``--release 2026-R1`` overrides ``ADIDT_CIM_RELEASE``. It initializes
``adi-linux`` and explicitly passes release, platform, jobs and output to make;
source both exports it emits only after successful preparation. It refuses to
overwrite an existing workspace. Reuse an existing manifest explicitly instead.
Its legacy 2023_R2 output is ``artifacts/<platform>/artifacts.json``; its
2026-R1 output is ``artifacts/2026_R1/<platform>/artifacts.json``. These explicit
consumer paths differ from CIM's release-aware defaults for 2023_R2.

See the consumer's
`hardware CI guide <https://github.com/analogdevicesinc/pyadi-dt/blob/main/doc/source/developer/hardware_ci.rst>`_
for runner configuration and preparation commands. Consumer deployment/version
availability is separate from this producer target; no consumer files are
installed or modified by CIM.

Integrity, cache failures and qualification boundaries
--------------------------------------------------------------------------------

See :doc:`/reference/adi-linux` for the artifact contract and cache policy.
A valid existing manifest is reused without compiling. A checksum/provenance
mismatch fails closed; do not edit provenance to make an old image acceptable.
Use a separate output directory, or deliberately invoke the helper with
``--force`` to rebuild. ``--force`` does not bypass archive checksum validation.
A corrupt archive must be explicitly removed before retrying. Keep the original
pinned helper available when verifying its existing artifacts: changing helper
bytes changes ``builder_sha256`` even if its apparent behavior is identical.

Offline fixture tests prove contracts, not cross-compilation. Successful real
builds and image validation prove artifact production, not board boot, radio
operation or hardware qualification. This target does not build DTBs, install
modules, construct root filesystems/BOOT.BIN, flash, reserve or test boards.
Patched overlay kernels and device-tree overlay workflows remain separate:
do not relabel their images as these ordinary checksum-pinned CIM kernels.
