# Reboot Recovery Flow Diagram

## Call Flow: Container Start After Reboot

```
┌─────────────────────────────────────────────────────────────────┐
│                       Docker Daemon                              │
│  (Container restart triggered by restart policy)                │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             │ POST /NetworkDriver.Join
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    main.rs: api_network_join()                   │
│  • Parse NetworkID, EndpointID, SandboxKey                      │
│  • Extract options                                              │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              manager.rs: endpoint_attach()                       │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐    │
│  │ 🆕 REBOOT RESILIENCE PHASE 1                           │    │
│  │                                                         │    │
│  │  1. Check if endpoint exists in memory                 │    │
│  │     endpoint_list.contains_key(&epuid)?                │    │
│  │                                                         │    │
│  │  2. If missing (post-reboot scenario):                 │    │
│  │     • Create new Endpoint::new(epuid)                  │    │
│  │     • Add to network's endpoint_list                   │    │
│  │     • Log: "Endpoint not found in memory, recreating" │    │
│  │                                                         │    │
│  │  3. Extract peer options from JSON                     │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              network.rs: endpoint_attach()                       │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐    │
│  │ 🆕 REBOOT RESILIENCE PHASE 2                           │    │
│  │                                                         │    │
│  │  1. Ensure network interface exists:                   │    │
│  │     ensure_network_interface_exists()                  │    │
│  │     ├─→ Check: network_interface_exists()?            │    │
│  │     └─→ If missing: ip link add vcan{id}              │    │
│  │                    ip link set up vcan{id}             │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐    │
│  │ 🆕 REBOOT RESILIENCE PHASE 3                           │    │
│  │                                                         │    │
│  │  2. Ensure endpoint interface exists:                  │    │
│  │     endpoint.ensure_interface_exists()                 │    │
│  │     ├─→ Check: interface_exists()?                    │    │
│  │     └─→ If missing: ip link add vxcan{id}             │    │
│  │                    ip link set up vxcan{id}            │    │
│  │     • Returns: Ok(true) if recreated                   │    │
│  │     • Log: "Successfully recreated interface pair"     │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                  │
│  ┌────────────────────────────────────────────────────────┐    │
│  │ STANDARD OPERATION (Enhanced)                          │    │
│  │                                                         │    │
│  │  3. Setup cangw rules:                                 │    │
│  │     • Network → Endpoint (bidirectional)               │    │
│  │     • For each peer endpoint:                          │    │
│  │       ├─→ 🆕 Validate peer interface exists           │    │
│  │       ├─→ If missing: Skip with warning               │    │
│  │       └─→ If exists: Create cross-rules               │    │
│  │           Peer → This (bidirectional)                  │    │
│  │                                                         │    │
│  │  4. Return JoinResponse with interface names           │    │
│  └────────────────────────────────────────────────────────┘    │
│                                                                  │
└────────────────────────────┬────────────────────────────────────┘
                             │
                             │ JoinResponse
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                       Docker Daemon                              │
│  • Moves peer interface to container namespace                  │
│  • Container starts successfully                                │
└─────────────────────────────────────────────────────────────────┘
```

## State Transitions

### Endpoint State Machine

```
                    Docker calls CreateEndpoint
                              │
                              ▼
                    ┌─────────────────┐
                    │  Endpoint        │
                    │  Created         │
                    │  (in memory)     │
                    └────────┬─────────┘
                             │
                    ┌────────┴─────────┐
                    │                  │
            Normal Start          After Reboot
                    │                  │
                    ▼                  ▼
         ┌─────────────────┐   ┌──────────────────┐
         │  Interface       │   │  Interface       │
         │  Exists          │   │  Missing         │
         │  (in kernel)     │   │  (kernel reset)  │
         └────────┬─────────┘   └────────┬─────────┘
                  │                      │
                  │               🆕 Auto-Recovery
                  │                      │
                  │              ┌───────▼─────────┐
                  │              │  Recreate       │
                  │              │  Interface      │
                  │              │  Transparently  │
                  │              └───────┬─────────┘
                  │                      │
                  └──────────┬───────────┘
                             │
                  Docker calls Join
                             │
                             ▼
                    ┌─────────────────┐
                    │  Endpoint        │
                    │  Attached        │
                    │  (cangw rules)   │
                    └─────────────────┘
```

## Error Handling Decision Tree

```
                    endpoint_attach() called
                             │
                             ▼
                    ┌─────────────────┐
                    │ Check Network    │
                    │ Interface        │
                    └────────┬─────────┘
                             │
                ┌────────────┴────────────┐
                │                         │
           Exists                    Missing
                │                         │
                ▼                         ▼
         ┌─────────────┐         ┌──────────────┐
         │ Continue     │         │ Recreate?    │
         └──────┬───────┘         └──────┬───────┘
                │                        │
                │              ┌─────────┴─────────┐
                │              │                   │
                │           Success            Failure
                │              │                   │
                │              ▼                   ▼
                │     ┌─────────────┐     ┌──────────────┐
                │     │ Continue     │     │ Return Error │
                │     └──────┬───────┘     │ Log Details  │
                │            │              └──────────────┘
                └────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Check Endpoint   │
                    │ Interface        │
                    └────────┬─────────┘
                             │
                ┌────────────┴────────────┐
                │                         │
           Exists                    Missing
                │                         │
                ▼                         ▼
         ┌─────────────┐         ┌──────────────┐
         │ Continue     │         │ Recreate?    │
         └──────┬───────┘         └──────┬───────┘
                │                        │
                │              ┌─────────┴─────────┐
                │              │                   │
                │           Success            Failure
                │              │                   │
                │              ▼                   ▼
                │     ┌─────────────┐     ┌──────────────┐
                │     │ Continue     │     │ Return Error │
                │     └──────┬───────┘     │ Log Details  │
                │            │              └──────────────┘
                └────────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Setup cangw      │
                    │ Rules            │
                    └────────┬─────────┘
                             │
                             ▼
                    ┌─────────────────┐
                    │ Return Success   │
                    └─────────────────┘
```

## Interface Lifecycle Comparison

### Before Reboot Resilience

```
Plugin Start          Container Start         System Reboot         Container Restart
     │                      │                       │                      │
     ▼                      │                       │                      │
┌─────────┐                │                       │                      │
│ Network │                │                       │                      │
│ Created │                │                       │                      │
└────┬────┘                │                       │                      │
     │                     │                       │                      │
     │  CreateEndpoint     │                       │                      │
     │◄────────────────────┘                       │                      │
     ▼                                              │                      │
┌─────────┐                                        │                      │
│Endpoint │                                        │                      │
│ Created │                                        │                      │
└────┬────┘                                        │                      │
     │                                              │                      │
     │  Join                                        │                      │
     │◄─────────────────────────────────────────┐  │                      │
     ▼                                          │  │                      │
┌─────────┐                                     │  │                      │
│ Attached│                                     │  │                      │
└────┬────┘                                     │  │                      │
     │                                          │  │                      │
     │  Container Running                       │  │                      │
     ▼                                          │  │                      │
┌─────────┐                                     │  ▼                      │
│  vxcan  │                                     │ REBOOT                  │
│  exists │                                     │  │                      │
└─────────┘                                     │  │  All interfaces      │
                                                │  │  destroyed!          │
                                                │  ▼                      │
                                                │ ┌──────────┐            │
                                                │ │ Plugin   │            │
                                                │ │ Restarts │            │
                                                │ │          │            │
                                                │ │ Memory   │            │
                                                │ │ Empty    │            │
                                                │ └────┬─────┘            │
                                                │      │                  │
                                                │      │  Join            │
                                                │      │◄─────────────────┘
                                                │      ▼
                                                │ ❌ FAILURE
                                                │ Endpoint not found
                                                │ Interface missing
```

### After Reboot Resilience

```
Plugin Start          Container Start         System Reboot         Container Restart
     │                      │                       │                      │
     ▼                      │                       │                      │
┌─────────┐                │                       │                      │
│ Network │                │                       │                      │
│ Created │                │                       │                      │
└────┬────┘                │                       │                      │
     │                     │                       │                      │
     │  CreateEndpoint     │                       │                      │
     │◄────────────────────┘                       │                      │
     ▼                                              │                      │
┌─────────┐                                        │                      │
│Endpoint │                                        │                      │
│ Created │                                        │                      │
└────┬────┘                                        │                      │
     │                                              │                      │
     │  Join                                        │                      │
     │◄─────────────────────────────────────────┐  │                      │
     ▼                                          │  │                      │
┌─────────┐                                     │  │                      │
│ Attached│                                     │  │                      │
└────┬────┘                                     │  │                      │
     │                                          │  │                      │
     │  Container Running                       │  │                      │
     ▼                                          │  │                      │
┌─────────┐                                     │  ▼                      │
│  vxcan  │                                     │ REBOOT                  │
│  exists │                                     │  │                      │
└─────────┘                                     │  │  All interfaces      │
                                                │  │  destroyed!          │
                                                │  ▼                      │
                                                │ ┌──────────┐            │
                                                │ │ Plugin   │            │
                                                │ │ Restarts │            │
                                                │ │          │            │
                                                │ │ Memory   │            │
                                                │ │ Empty    │            │
                                                │ └────┬─────┘            │
                                                │      │                  │
                                                │      │  Join            │
                                                │      │◄─────────────────┘
                                                │      ▼
                                                │ 🆕 RECOVERY
                                                │      │
                                                │      ├─→ Recreate endpoint
                                                │      ├─→ Recreate network ifc
                                                │      ├─→ Recreate vxcan pair
                                                │      └─→ Setup cangw rules
                                                │      │
                                                │      ▼
                                                │ ✅ SUCCESS
                                                │ Container starts
```

## Code Flow Summary

### New Functions Called During Recovery

```
api_network_join()
  └─→ NetworkManager::endpoint_attach()
       │
       ├─→ 🆕 Check: endpoint_list.contains_key()?
       ├─→ 🆕 If missing: Endpoint::new()           [endpoint.rs]
       │
       └─→ Network::endpoint_attach()
            │
            ├─→ 🆕 ensure_network_interface_exists() [network.rs]
            │    ├─→ network_interface_exists()     [network.rs]
            │    └─→ ip link add/set                [system call]
            │
            ├─→ 🆕 Endpoint::ensure_interface_exists() [endpoint.rs]
            │    ├─→ interface_exists()              [endpoint.rs]
            │    └─→ ip link add/set                 [system call]
            │
            ├─→ add_cangw_rule()                     [existing]
            └─→ 🆕 Peer validation loop              [network.rs]
                 └─→ interface_exists() for peers     [endpoint.rs]
```

## Performance Timeline

### Cold Start (Post-Reboot)

```
Time: 0ms      10ms     20ms    30ms    40ms    50ms    60ms
      │────────│────────│───────│───────│───────│───────│
      │
      ├─→ Join called
      │
      ├─→ Check network interface (~1ms)
      │   └─→ Missing: recreate (~15ms)
      │
      ├─→ Check endpoint interface (~1ms)
      │   └─→ Missing: recreate (~15ms)
      │
      ├─→ Check peer interfaces (~1ms)
      │
      ├─→ Create cangw rules (~5ms)
      │
      └─→ Return success
          │
          Total: ~40-60ms (one-time cost)
```

### Warm Start (Normal Operation)

```
Time: 0ms      10ms     20ms
      │────────│────────│
      │
      ├─→ Join called
      │
      ├─→ Check network interface (~1ms)
      │   └─→ Exists: continue
      │
      ├─→ Check endpoint interface (~1ms)
      │   └─→ Exists: continue
      │
      ├─→ Check peer interfaces (~1ms)
      │
      ├─→ Create cangw rules (~5ms)
      │
      └─→ Return success
          │
          Total: ~10ms (no recreation needed)
```

## Key Design Decisions

1. **Lazy Recovery** - Interfaces recreated on-demand, not proactively
   - Pros: Fast plugin startup, only recreates what's needed
   - Cons: Slight delay on first container start post-reboot

2. **Idempotent Operations** - Can be called multiple times safely
   - Handles concurrent container starts
   - Graceful handling of race conditions

3. **Fail-Fast for Unrecoverable Errors** - Return errors immediately
   - Network not found → Error (can't recover)
   - Interface creation fails → Error with details
   - Command execution fails → Error with context

4. **Transparent Recovery** - No user intervention required
   - No new configuration options
   - No docker-compose.yml changes
   - Works with existing networks

5. **Comprehensive Logging** - Every recovery action logged
   - Easy troubleshooting
   - Clear audit trail
   - Helpful error messages

