# QUASAR-SUBNET: Comprehensive Miner Domination Strategy

## 1. What Is QUASAR-SUBNET?

QUASAR-SUBNET (Subnet 24 on Bittensor, netuid 383 on testnet) is a **long-context kernel optimization and inference verification subnet** built by SILX Labs. Its core mission: optimize CUDA kernels for the **Quasar attention mechanism** — a linear-time alternative to standard multi-head attention that uses Hierarchical Flow Anchoring (HFA) instead of positional embeddings.

Miners fork the `troy12x/flash-linear-attention` repository, optimize Triton/CUDA kernels (primarily `chunk.py`), benchmark their optimizations, and submit results. Validators clone the fork, independently re-run the benchmarks, verify the code, and assign scores.

**The subnet is essentially a decentralized CUDA kernel optimization competition.**

---

## 2. How the Mining Mechanism Works

### 2.1 The Mining Loop

1. **Fork** the target repo: `https://github.com/troy12x/flash-linear-attention.git`
2. **Load an LLM** (default: `Qwen/Qwen3-4B-Instruct-2507`) to act as a code-optimization agent
3. **Read target files** from `fla/ops/quasar/`:
   - `chunk.py` (PRIMARY target)
   - `chunk_intra_token_parallel.py`
   - `forward_substitution.py`
   - `fused_recurrent.py`
   - `gate.py`
   - `__init__.py`
4. **Generate optimized kernel code** using the LLM agent with full repo context
5. **Run benchmark tests** at sequence lengths: `[512, 1024, 2048, 4096, 16384, 65536, 100000]`
6. **Submit results** (fork URL, commit hash, tokens/sec, VRAM usage) to the Validator API
7. **Repeat** for `AGENT_ITERATIONS` cycles (default 100)

### 2.2 BYOC Mode (Bring Your Own Code)

Instead of relying on the LLM agent, miners can provide a **hand-optimized kernel** via `BYOC_FILE_PATH`. The LLM then uses this as a reference/expert code to adapt to the repository structure. **This is the key to domination.**

```bash
export BYOC_FILE_PATH=./my_optimized_kernels/chunk.py
export TARGET_SEQUENCE_LENGTH=100000
./START_MINER.sh
```

---

## 3. How Scoring Works (Critical)

There are **two scoring systems** running in parallel:

### 3.1 Kernel Performance Scoring (PRIMARY — what determines your rewards)

The validator:
1. **Clones your fork** and checks out your commit
2. **Validates imports** (HARD GATE — fail = score 0.0)
3. **Runs benchmark tests** at sequence lengths `[512, 1024, 2048, target_seq_len]`
4. **Calculates score**:
   - If `actual_performance >= claimed_performance * 0.9` → score = `1.0 + (actual - claimed) / claimed`
   - If `actual_performance < claimed_performance * 0.9` → score = **0.0**

**Translation: your claimed tokens/sec must match reality within 10%. Then, HIGHER actual throughput = HIGHER score.**

### 3.2 Inference Verification (Anti-Cheat Gate)

Validators also run logit verification to ensure miners aren't faking results:
1. Generate random prompt tokens
2. Run inference on both the miner's container and a reference model
3. Compare logits at a random decode step
4. Requirements:
   - **Cosine similarity >= 0.99**
   - **Max absolute difference <= 0.1**
5. **Fail = score infinity (rejected)**

### 3.3 Weight Assignment: Winner-Take-All

The subnet uses **epsilon-dominance winner-take-all**: the single best-performing verified miner gets **100% of the weight**. Everyone else gets 0%.

```
leader = miner with lowest score (1/throughput), verified, earliest block
weights[leader] = 1.0
weights[everyone_else] = 0.0
```

**This means second place gets NOTHING. You must be #1.**

---

## 4. Import Requirements (Instant-Fail Gate)

### MANDATORY imports in chunk.py (missing ANY = score 0.0):
```python
from fla.utils import autocast_custom_bwd
from fla.utils import autocast_custom_fwd
from fla.utils import autotune_cache_kwargs
from fla.utils import check_shared_mem
from fla.utils import input_guard
```

### FORBIDDEN imports (presence = score 0.0):
```python
from fla.ops.gla   # FORBIDDEN
from fla.ops.kda   # FORBIDDEN
```

---

## 5. Strategy to Dominate

### 5.1 The Winning Formula

Since this is **winner-take-all on kernel throughput**, the strategy is straightforward:

**Write the fastest possible Triton/CUDA kernel for the Quasar attention mechanism at 100K sequence length.**

### 5.2 Specific Optimization Targets

The benchmark test creates a `QuasarAttention` layer with:
- `batch_size = 1`
- `seq_len = 100,000` (target)
- `hidden_size = 512`
- `head_dim = 64`
- `num_heads = 8`
- `mode = "chunk"`
- Uses `torch.autocast` with `bfloat16`

The score is **tokens/sec** = `(batch_size * seq_len * num_runs) / elapsed_time`.

### 5.3 Kernel Optimization Techniques

To maximize throughput at 100K sequence length:

1. **Optimize the chunk-wise Triton kernels**:
   - Tune block sizes for your GPU (autotune)
   - Maximize occupancy and minimize register pressure
   - Use shared memory efficiently (`check_shared_mem` is required)
   - Optimize memory access patterns (coalesced reads/writes)

2. **Reduce memory overhead**:
   - Remove unnecessary intermediate tensor allocations
   - Use in-place operations where possible
   - Fuse operations (the gate kernel in `gate.py` should be integrated)
   - Use the `fused_recurrent.py` path efficiently

3. **Leverage `chunk_intra_token_parallel.py`**:
   - This file handles intra-token parallelism within chunks
   - Optimizing chunk size and parallelization strategy is critical at 100K tokens

4. **Use BYOC mode with hand-optimized kernels**:
   - Don't rely on the LLM agent to generate optimal code
   - Write kernels by hand using Triton best practices
   - Test extensively at the target sequence length
   - Set `BYOC_FILE_PATH` to your optimized code

### 5.4 Hardware Recommendations

- **GPU**: A100 80GB or H100 (recommended for 100K sequence length)
- **VRAM**: 24GB minimum, 40-80GB recommended for long sequences
- **CUDA Compute**: 7.0+ (A100 = 8.0, H100 = 9.0)

### 5.5 Operational Best Practices

1. **Always include all required imports** — missing one = instant zero
2. **Never use forbidden imports** (`fla.ops.gla`, `fla.ops.kda`)
3. **Match function signatures exactly** — export `chunk_quasar` correctly
4. **Claim performance accurately** — over-claiming by >10% = zero score
5. **Run the inference server** for logit verification — use the same reference model (`Qwen/Qwen3-4B-Instruct-2507`)
6. **Submit early** — ties are broken by earliest block number

### 5.6 The Underclaimiing Strategy

Since score = `1.0 + (actual - claimed) / claimed`, you can **underclaim** your performance:
- If your kernel does 50,000 tokens/sec, claim 40,000
- Validator measures 50,000 → score = 1.0 + (50000-40000)/40000 = **1.25**
- This is better than claiming 50,000 → score = 1.0

However, the validator sorts by **actual performance** for winner-take-all, so underclaiming only helps avoid the 10% tolerance gate. The actual throughput is what matters for leadership.

---

## 6. SILX-AI HuggingFace Models (Owner's Work)

SILX AI publishes models on [HuggingFace](https://huggingface.co/silx-ai):

| Model | Architecture | Description |
|-------|-------------|-------------|
| **Quasar-3.0-Instract-v2** | 7B params (distilled from 400B) | Uses Token Temperature Mechanism (TTM) for reasoning/contextual focus |
| **Quasar-3.0-Final** | 7B | Base model with community quantizations (GGUF, GPTQ) |
| **QuasarV4-LNNs** | Various | Non-transformer models using Liquid Neural Networks |

### Key Architectural Innovation: Token Temperature Mechanism (TTM)
- Classifies tokens as "hot" (critical) or "cold" (less relevant)
- Modulates attention based on token importance
- Removes traditional positional embeddings
- Replaced with HFA (Hierarchical Flow Anchoring) for linear-time attention

### What This Means for Miners
The owner is training models that USE the Quasar attention mechanism. The subnet exists to **crowdsource kernel optimization** for these models. If you optimize the kernels well, you're directly improving the performance of their production models.

Understanding the HFA/Flow Attention architecture helps you write better kernels because you know:
- The attention is **linear-time** (not quadratic)
- There are no positional embeddings
- The mechanism uses bidirectional flow dynamics
- Chunk-based processing is fundamental to the architecture

---

## 7. Summary: The Domination Playbook

1. **Study** the `troy12x/flash-linear-attention` repo deeply, especially `fla/ops/quasar/`
2. **Write** hand-optimized Triton kernels for `chunk.py` targeting 100K sequence length
3. **Use BYOC mode** — don't rely on the LLM agent
4. **Benchmark** extensively on your target GPU before submitting
5. **Include all required imports** — no exceptions
6. **Run the inference server** on the correct reference model
7. **Submit early** — earliest block wins ties
8. **Iterate** — the miner supports continuous optimization loops
9. **Target the bottleneck** — profile to find what's slow at 100K tokens (likely memory bandwidth and chunk coordination)
10. **Use an A100/H100** — raw GPU power matters in a throughput competition
