// The launch of one mesh draw by the card (`launch.rs`): an amplification
// shader of ours in front of every translated mesh shader when
// VICTORY_MESH_LAUNCH is on. It reads the title's record where the title's
// passes wrote it, bounds it as the clamp of `indirect.rs` bounds a mesh
// record, charges the list's budget the same way, and launches the mesh
// groups. Nothing runs between the draws: no pass of ours, no wait for every
// earlier shader write. (From fxceless b1d777a8; after Kelvaroq patch 0041.)
//
// Compiled when the first mesh pipeline is made, by the PC's dxc.exe
// (`launch::compile`), with REST defined as the clamp's (`indirect::rest`).

#ifndef REST
#define REST 1
#endif

// The payload every translated mesh shader declares and never reads
// (`victory_shader::emit::mesh_payload_bytes`, 4 bytes with the switch on).
struct Payload {
    uint unused;
};
groupshared Payload payload;

// `direct`: the group count of a mesh dispatch the CPU knows (the title's
// DispatchMesh), or 0xffffffff for a record the card reads at `record`.
cbuffer Launch : register(b0, space9) {
    uint direct;
};
// The record: three words, x, y, z (the console's mesh dispatches are
// one-dimensional).
ByteAddressBuffer record : register(t0, space9);
// The list's work so far in its first word; then, as the clamp leaves them
// (read back when the list has run): the largest group count a record asked
// for, the groups asked in all, the records refused for want of budget, the
// largest cost there was.
RWByteAddressBuffer budget : register(u0, space9);

static const uint BUDGET = 1u << 27;
static const uint DIRECT_NONE = 0xffffffffu;

[numthreads(1, 1, 1)]
void main() {
    uint3 g = uint3(0, 0, 0);
    if (direct != DIRECT_NONE) {
        // Known on the CPU, as before: no budget, the host's limit only.
        g = uint3(min(direct, 65535u), 1u, 1u);
    } else {
        uint3 asked = record.Load3(0);
        // A record over the most a dispatch runs is a slot the title has
        // not filled (it leaves all ones there): empty, and not counted.
        if (asked.x > 65535u) {
            asked = uint3(0, 0, 0);
        }
        g = uint3(asked.x, min(asked.y, 1u), min(asked.z, 1u));
        uint cost = g.x * g.y * g.z * 128u;
        // As the clamp charges one record: a record that does not fit whole
        // runs empty and (REST) takes the rest of the budget.
        bool fit = true;
        if (cost > 0) {
            uint before;
            budget.InterlockedOr(0, 0, before);
            [allow_uav_condition]
            while (before < BUDGET) {
                uint next = (BUDGET - before < cost) ? (REST ? BUDGET : before) : (before + cost);
                if (next == before) {
                    break;
                }
                uint prev;
                budget.InterlockedCompareExchange(0, before, next, prev);
                if (prev == before) {
                    break;
                }
                before = prev;
            }
            fit = before < BUDGET && BUDGET - before >= cost;
            uint told;
            if (!fit) {
                budget.InterlockedAdd(12, 1u, told);
            }
            if (cost >= (1u << 22)) {
                budget.InterlockedMax(16, cost, told);
            }
        }
        if (!fit) {
            g = uint3(0, 0, 0);
        }
        uint was;
        budget.InterlockedMax(4, asked.x, was);
        budget.InterlockedAdd(8, asked.x, was);
    }
    payload.unused = 0;
    DispatchMesh(g.x, g.y, g.z, payload);
}
