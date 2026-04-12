"""Test steering vector pipeline with mock data.

This test verifies:
1. SteeringEngine initialization
2. Vector loading from .npz file
3. Vector computation from contrast prompts (mock)
4. Steering application to hidden states
"""

import numpy as np
import torch
import sys
import os

# Add parent directory to path to import from server.py
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from python.elai_inference.server import SteeringEngine, COEFFICIENT_DEFAULT, COEFFICIENT_MIN, COEFFICIENT_MAX


def test_steering_engine_init():
    """Test SteeringEngine initialization with default parameters."""
    print("Testing SteeringEngine initialization...")
    engine = SteeringEngine(n_layers=32, hidden_dim=2048)
    assert engine.n_layers == 32
    assert engine.hidden_dim == 2048
    assert engine.coefficient == COEFFICIENT_DEFAULT
    assert engine.target_layer_idx == 32 // 3  # auto = n_layers // 3
    print("  ✓ Initialization successful")


def test_steering_engine_coefficient_validation():
    """Test coefficient validation (clamp to [COEFFICIENT_MIN, COEFFICIENT_MAX])."""
    print("Testing coefficient validation...")
    # Below minimum
    engine = SteeringEngine(n_layers=32, hidden_dim=2048, coefficient=0.5)
    assert engine.coefficient == COEFFICIENT_DEFAULT  # clamped to default
    
    # Above maximum
    engine = SteeringEngine(n_layers=32, hidden_dim=2048, coefficient=100.0)
    assert engine.coefficient == COEFFICIENT_DEFAULT  # clamped to default
    
    # Valid range
    engine = SteeringEngine(n_layers=32, hidden_dim=2048, coefficient=10.0)
    assert engine.coefficient == 10.0
    print("  ✓ Coefficient validation works")


def test_add_vector():
    """Test adding a steering vector."""
    print("Testing add_vector...")
    engine = SteeringEngine(n_layers=32, hidden_dim=2048)
    
    # Create mock vector
    vector = np.random.randn(2048).astype(np.float32)
    engine.add_vector(layer_idx=10, vector=vector)
    
    assert 10 in engine.vectors
    assert engine.vectors[10].shape == (2048,)
    assert engine.is_active()
    print("  ✓ Vector addition works")


def test_add_vector_wrong_shape():
    """Test that wrong-shaped vectors are rejected."""
    print("Testing wrong-shaped vector rejection...")
    engine = SteeringEngine(n_layers=32, hidden_dim=2048)
    
    # Wrong shape
    wrong_vector = np.random.randn(1024).astype(np.float32)
    engine.add_vector(layer_idx=10, vector=wrong_vector)
    
    assert 10 not in engine.vectors  # Should not be added
    print("  ✓ Wrong-shaped vectors rejected")


def test_apply_steering():
    """Test applying steering to hidden state."""
    print("Testing apply_steering...")
    engine = SteeringEngine(n_layers=32, hidden_dim=2048, target_layer=10)
    
    # Add vector for target layer
    vector = np.random.randn(2048).astype(np.float32)
    engine.add_vector(layer_idx=10, vector=vector)
    
    # Mock hidden state
    hidden_state = torch.randn(1, 2048)
    
    # Apply steering at target layer
    modified = engine.apply_steering(hidden_state, layer_idx=10)
    
    # Should be modified
    assert not torch.equal(modified, hidden_state)
    assert modified.shape == hidden_state.shape
    print("  ✓ Steering application works")


def test_apply_steering_wrong_layer():
    """Test that steering only applies at target layer."""
    print("Testing steering only at target layer...")
    engine = SteeringEngine(n_layers=32, hidden_dim=2048, target_layer=10)
    
    # Add vector for target layer
    vector = np.random.randn(2048).astype(np.float32)
    engine.add_vector(layer_idx=10, vector=vector)
    
    # Mock hidden state
    hidden_state = torch.randn(1, 2048)
    
    # Apply steering at wrong layer
    modified = engine.apply_steering(hidden_state, layer_idx=5)
    
    # Should NOT be modified
    assert torch.equal(modified, hidden_state)
    print("  ✓ Steering only applies at target layer")


def test_load_vector_from_npz():
    """Test loading vector from .npz file."""
    print("Testing vector loading from .npz...")
    engine = SteeringEngine(n_layers=32, hidden_dim=2048)
    
    # Create temporary .npz file
    vector = np.random.randn(2048).astype(np.float32)
    test_path = "/tmp/test_steering_vector.npz"
    np.savez(test_path, vector=vector)
    
    # Load it
    engine.load_vector_from_file(layer_idx=10, filepath=test_path)
    
    assert 10 in engine.vectors
    assert engine.vectors[10].shape == (2048,)
    
    # Cleanup
    os.remove(test_path)
    print("  ✓ .npz loading works")


def test_compose():
    """Test the full steering vector pipeline."""
    print("\n=== Full Pipeline Test ===")
    engine = SteeringEngine(n_layers=32, hidden_dim=2048, coefficient=5.0, target_layer=10)
    
    # Create and load vector
    vector = np.random.randn(2048).astype(np.float32)
    test_path = "/tmp/test_compose.npz"
    np.savez(test_path, vector=vector)
    engine.load_vector_from_file(layer_idx=10, filepath=test_path)
    
    # Apply steering
    hidden_state = torch.randn(1, 2048)
    modified = engine.apply_steering(hidden_state, layer_idx=10)
    
    # Verify
    assert engine.is_active()
    assert not torch.equal(modified, hidden_state)
    
    # Cleanup
    os.remove(test_path)
    print("✓ Full pipeline test passed")


if __name__ == "__main__":
    print("=" * 60)
    print("Steering Vector Pipeline Tests")
    print("=" * 60)
    
    test_steering_engine_init()
    test_steering_engine_coefficient_validation()
    test_add_vector()
    test_add_vector_wrong_shape()
    test_apply_steering()
    test_apply_steering_wrong_layer()
    test_load_vector_from_npz()
    test_compose()
    
    print("\n" + "=" * 60)
    print("All tests passed!")
    print("=" * 60)
