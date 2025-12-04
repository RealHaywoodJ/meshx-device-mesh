#!/usr/bin/env python3
"""
MeshX Network Simulation
Prototype for testing mesh network dynamics
Copyright (c) 2025 MeshX Foundation
"""

import random
import time
import hashlib
from dataclasses import dataclass
from typing import List, Dict, Tuple

@dataclass
class MeshXNode:
    """Represents a node in the MeshX network"""
    node_id: str
    location: Tuple[float, float]  # (latitude, longitude)
    resources: Dict[str, int]  # cpu, ram, storage, bandwidth
    meshx_balance: float
    is_validator: bool = False
    
    def __post_init__(self):
        self.shard = self.assign_shard()
    
    def assign_shard(self) -> str:
        """Assign node to continental shard based on location"""
        lat, lon = self.location
        
        if 15 < lat < 75 and -170 < lon < -50:
            return "North America"
        elif 35 < lat < 75 and -15 < lon < 40:
            return "Europe"
        elif -10 < lat < 55 and 40 < lon < 150:
            return "Asia"
        elif -60 < lat < 15 and -85 < lon < -30:
            return "South America"
        elif -40 < lat < 40 and -20 < lon < 55:
            return "Africa"
        elif -50 < lat < -10 and 110 < lon < 180:
            return "Oceania"
        elif lat < -60:
            return "Antarctica"
        else:
            return "North America"  # Default

class MeshXSimulator:
    """Simulates the MeshX network"""
    
    def __init__(self, num_nodes: int = 1000):
        self.nodes = []
        self.epoch = 0
        self.total_compute_jobs = 0
        self.meshx_price = 0.10  # Starting price in USD
        
        print(f"🌐 Initializing MeshX simulation with {num_nodes} nodes...")
        self._generate_nodes(num_nodes)
    
    def _generate_nodes(self, count: int):
        """Generate random nodes across the globe"""
        for i in range(count):
            # Random global distribution
            lat = random.uniform(-90, 90)
            lon = random.uniform(-180, 180)
            
            node = MeshXNode(
                node_id=f"node_{i:04d}",
                location=(lat, lon),
                resources={
                    'cpu': random.randint(2, 16),
                    'ram': random.choice([4, 8, 16, 32, 64]),
                    'storage': random.choice([100, 500, 1000, 2000]),
                    'bandwidth': random.randint(10, 1000)
                },
                meshx_balance=random.uniform(100, 10000)
            )
            self.nodes.append(node)
    
    def select_validators(self, count: int = 100) -> List[MeshXNode]:
        """Select validators using simplified PoP²"""
        # Sort by VRF output (simulated with hash)
        epoch_seed = hashlib.sha256(str(self.epoch).encode()).digest()
        
        scored_nodes = []
        for node in self.nodes:
            # VRF simulation
            vrf_input = epoch_seed + node.node_id.encode()
            vrf_output = int(hashlib.sha256(vrf_input).hexdigest(), 16)
            
            # Weight by stake
            score = vrf_output * node.meshx_balance
            scored_nodes.append((score, node))
        
        # Select top nodes
        scored_nodes.sort(key=lambda x: x[0], reverse=True)
        validators = [node for _, node in scored_nodes[:count]]
        
        # Mark as validators
        for node in self.nodes:
            node.is_validator = node in validators
        
        return validators
    
    def simulate_compute_job(self) -> Dict[str, any]:
        """Simulate a compute job execution"""
        # Select random nodes for job
        available_nodes = [n for n in self.nodes if n.resources['cpu'] >= 2]
        selected = random.sample(available_nodes, min(10, len(available_nodes)))
        
        # Calculate job cost
        compute_units = random.randint(100, 10000)
        meshx_cost = compute_units * 0.001  # 0.001 MESHX per unit
        usd_cost = meshx_cost * self.meshx_price
        
        # Distribute rewards
        reward_per_node = meshx_cost / len(selected)
        for node in selected:
            node.meshx_balance += reward_per_node
        
        self.total_compute_jobs += 1
        
        return {
            'job_id': f"job_{self.total_compute_jobs:06d}",
            'compute_units': compute_units,
            'meshx_cost': meshx_cost,
            'usd_cost': usd_cost,
            'nodes_used': len(selected),
            'avg_latency_ms': random.randint(10, 100)
        }
    
    def run_epoch(self):
        """Run one epoch of the simulation"""
        self.epoch += 1
        print(f"\n🔄 Epoch {self.epoch}")
        
        # Select validators
        validators = self.select_validators()
        print(f"   Validators selected: {len(validators)}")
        
        # Count nodes by shard
        shard_counts = {}
        for node in self.nodes:
            shard_counts[node.shard] = shard_counts.get(node.shard, 0) + 1
        
        print("   Shard distribution:")
        for shard, count in sorted(shard_counts.items()):
            print(f"      {shard}: {count} nodes")
        
        # Simulate compute jobs
        jobs_this_epoch = random.randint(50, 200)
        total_meshx_spent = 0
        
        for _ in range(jobs_this_epoch):
            job = self.simulate_compute_job()
            total_meshx_spent += job['meshx_cost']
        
        print(f"   Compute jobs executed: {jobs_this_epoch}")
        print(f"   Total MESHX transacted: {total_meshx_spent:.2f}")
        print(f"   Network value: ${total_meshx_spent * self.meshx_price:.2f}")
        
        # Update MESHX price based on activity
        self.meshx_price *= random.uniform(0.98, 1.02)
        print(f"   MESHX price: ${self.meshx_price:.4f}")
    
    def get_network_stats(self) -> Dict[str, any]:
        """Get current network statistics"""
        total_resources = {
            'cpu': sum(n.resources['cpu'] for n in self.nodes),
            'ram': sum(n.resources['ram'] for n in self.nodes),
            'storage': sum(n.resources['storage'] for n in self.nodes),
            'bandwidth': sum(n.resources['bandwidth'] for n in self.nodes)
        }
        
        total_meshx = sum(n.meshx_balance for n in self.nodes)
        
        return {
            'total_nodes': len(self.nodes),
            'total_validators': sum(1 for n in self.nodes if n.is_validator),
            'total_resources': total_resources,
            'total_meshx_supply': total_meshx,
            'market_cap_usd': total_meshx * self.meshx_price,
            'total_compute_jobs': self.total_compute_jobs
        }
    
    def print_summary(self):
        """Print network summary"""
        stats = self.get_network_stats()
        
        print("\n" + "="*50)
        print("📊 MESHX NETWORK SUMMARY")
        print("="*50)
        print(f"Total Nodes: {stats['total_nodes']:,}")
        print(f"Active Validators: {stats['total_validators']}")
        print(f"Total CPU Cores: {stats['total_resources']['cpu']:,}")
        print(f"Total RAM: {stats['total_resources']['ram']:,} GB")
        print(f"Total Storage: {stats['total_resources']['storage']:,} GB")
        print(f"Total Bandwidth: {stats['total_resources']['bandwidth']:,} Mbps")
        print(f"Total MESHX Supply: {stats['total_meshx_supply']:,.2f}")
        print(f"Market Cap: ${stats['market_cap_usd']:,.2f}")
        print(f"Compute Jobs Executed: {stats['total_compute_jobs']:,}")
        print("="*50)

def main():
    """Run the simulation"""
    print("🚀 MeshX Network Simulator")
    print("="*50)
    
    # Create simulator
    sim = MeshXSimulator(num_nodes=10000)
    
    # Run simulation
    print("\n⏰ Running simulation for 10 epochs...")
    for _ in range(10):
        sim.run_epoch()
        time.sleep(1)  # Pause for readability
    
    # Final summary
    sim.print_summary()
    
    print("\n✅ Simulation complete!")
    print("💡 This demonstrates how MeshX will scale to billions of devices")

if __name__ == "__main__":
    main()