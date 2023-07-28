# Setup

Install rust-lang kit with rustup: 

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install libtorch under /opt/: 

```bash
cd /opt/
sudo wget https://download.pytorch.org/libtorch/cu117/libtorch-cxx11-abi-shared-with-deps-2.0.1%2Bcu117.zip
sudo unzip libtorch-cxx11-abi-shared-with-deps-2.0.1+cu117.zip 
```

Operating System: `Ubuntu-22.04`

Here I list my hardware details:  
```
# CPU INFO
Architecture:            x86_64
  CPU op-mode(s):        32-bit, 64-bit
  Address sizes:         48 bits physical, 48 bits virtual
  Byte Order:            Little Endian
CPU(s):                  16
  On-line CPU(s) list:   0-15
Vendor ID:               AuthenticAMD
  Model name:            AMD Ryzen 7 5800H with Radeon Graphics
    CPU family:          25
    Model:               80
    Thread(s) per core:  2
    Core(s) per socket:  8
    Socket(s):           1
    Stepping:            0
    BogoMIPS:            6387.85
    Flags:               fpu vme de pse tsc msr pae mce cx8 apic sep mtrr pge mca cmov pat pse36 clflush mmx fxsr sse sse2 ht syscall nx mmxext fxsr_opt pdpe1gb rdtscp lm constant
                         _tsc rep_good nopl tsc_reliable nonstop_tsc cpuid extd_apicid pni pclmulqdq ssse3 fma cx16 sse4_1 sse4_2 movbe popcnt aes xsave avx f16c rdrand hypervisor
                          lahf_lm cmp_legacy svm cr8_legacy abm sse4a misalignsse 3dnowprefetch osvw topoext perfctr_core ssbd ibrs ibpb stibp vmmcall fsgsbase bmi1 avx2 smep bmi2
                          erms invpcid rdseed adx smap clflushopt clwb sha_ni xsaveopt xsavec xgetbv1 xsaves clzero xsaveerptr arat npt nrip_save tsc_scale vmcb_clean flushbyasid
                         decodeassists pausefilter pfthreshold v_vmsave_vmload umip vaes vpclmulqdq rdpid fsrm
Virtualization features:
  Virtualization:        AMD-V
  Hypervisor vendor:     Microsoft
  Virtualization type:   full
Caches (sum of all):
  L1d:                   256 KiB (8 instances)
  L1i:                   256 KiB (8 instances)
  L2:                    4 MiB (8 instances)
  L3:                    16 MiB (1 instance)
Vulnerabilities:
  Itlb multihit:         Not affected
  L1tf:                  Not affected
  Mds:                   Not affected
  Meltdown:              Not affected
  Mmio stale data:       Not affected
  Retbleed:              Not affected
  Spec store bypass:     Mitigation; Speculative Store Bypass disabled via prctl and seccomp
  Spectre v1:            Mitigation; usercopy/swapgs barriers and __user pointer sanitization
  Spectre v2:            Mitigation; Retpolines, IBPB conditional, IBRS_FW, STIBP conditional, RSB filling, PBRSB-eIBRS Not affected
  Srbds:                 Not affected
  Tsx async abort:       Not affected

# GPU INFO
Wed Jul 26 00:17:39 2023
+-----------------------------------------------------------------------------+
| NVIDIA-SMI 525.85.05    Driver Version: 528.24       CUDA Version: 12.0     |
|-------------------------------+----------------------+----------------------+
| GPU  Name        Persistence-M| Bus-Id        Disp.A | Volatile Uncorr. ECC |
| Fan  Temp  Perf  Pwr:Usage/Cap|         Memory-Usage | GPU-Util  Compute M. |
|                               |                      |               MIG M. |
|===============================+======================+======================|
|   0  NVIDIA GeForce ...  On   | 00000000:01:00.0  On |                  N/A |
| N/A   51C    P8    13W /  95W |   1531MiB /  6144MiB |      3%      Default |
|                               |                      |                  N/A |
+-------------------------------+----------------------+----------------------+

+-----------------------------------------------------------------------------+
| Processes:                                                                  |
|  GPU   GI   CI        PID   Type   Process name                  GPU Memory |
|        ID   ID                                                   Usage      |
|=============================================================================|
|    0   N/A  N/A        23      G   /Xwayland                       N/A      |
+-----------------------------------------------------------------------------+
```

# To Run

```shell
LD_LIBRARY_PATH=/opt/libtorch/lib:$LD_LIBRARY_PATH cargo run
```

# ROADMAP

Who can take an action in this round? (finished)
- the attacker moves first by calling a function (with computed input) in defender code
- the defender moves by executing **checks** on some OP\_CODE positions
	- if some check is failed: current execution is reverted
	- if some attacker function is called: 
		- then it comes to attacker's round (from the called function)
	- if defender reaches a return statement in a function: 
		- then it comes to attacker's round (from the code position where the attacker called last function)
- when the attacker reaches a return statement in a function: 
	- then it comes to defender's round (from the code position where the defender called last function)
	- specially, the game is over if this function is not called by the defender

Some requirements on utility function: (finished)
- defender: 
	- ban attackers effectively
	- on pre-written test cases
		- execute correctly, don't ban them
		- less check operations and less memory cost
- attacker: 
	- less operations and less memory
	- lingering in the defender's code longer
	- can steal money effectively

Formulate the utility function: (wip)
$$
	u_{\mathscr{A}}(h) = (defender\_state(h).balance = 0)
	u_{\mathscr{D}}(h) = (defender\_state(h).balance \ne 0) \wedge (defender\_tests(h).passed \ne 0)
$$

Make the action space easier to learn:
- build operator tree instead of direct computation
- sample by probability

Trivial implementations: (wip)
- implement an operator tree for the attacker
- implement check triggering when a piece of defender code is executed
- implement auxiliary states
- compute gas fee for the host
