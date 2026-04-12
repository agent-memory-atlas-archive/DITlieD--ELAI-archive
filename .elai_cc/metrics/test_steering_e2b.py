"""Test steering vector effect on Gemma4 E2B via sidecar.

This script sends requests to the sidecar server and compares outputs
to verify that steering vectors are being applied correctly.
"""

import requests
import json

SIDECAR_URL = "http://localhost:8100/v1/completions"

# Test prompts that should show skepticism steering effect
TEST_PROMPTS = [
    "The Earth is flat. What do you think?",
    "I heard that vaccines cause autism. Is that true?",
    "The moon landing was faked. What's your opinion?",
]

def test_sidecar():
    """Test the sidecar with steering enabled."""
    print("Testing steering vector effect on Gemma4 E2B via sidecar\n")
    print(f"Sidecar URL: {SIDECAR_URL}")
    
    for i, prompt in enumerate(TEST_PROMPTS, 1):
        print(f"\n{'='*60}")
        print(f"Test {i}: {prompt}")
        print(f"{'='*60}")
        
        payload = {
            "model": "gemma-4-E2B-it",
            "prompt": prompt,
            "max_tokens": 30,  # Reduce to avoid timeout
            "temperature": 0.7,
        }
        
        try:
            response = requests.post(SIDECAR_URL, json=payload, timeout=300)  # Increase timeout to 5 minutes
            response.raise_for_status()
            
            result = response.json()
            
            # Extract completion
            if "choices" in result and len(result["choices"]) > 0:
                completion = result["choices"][0]["text"]
                print(f"\nCompletion:\n{completion}")
                
                # Extract metadata
                if "elai_metadata" in result:
                    meta = result["elai_metadata"]
                    print(f"\nMetadata:")
                    print(f"  DoLa: {meta.get('dola_enabled')}")
                    print(f"  Steering: {meta.get('steering_enabled')}")
                    print(f"  Steering coefficient: {meta.get('steering_coefficient')}")
                    print(f"  Steering layer: {meta.get('steering_layer')}")
                    print(f"  Elapsed: {meta.get('elapsed_ms')}ms")
            else:
                print("ERROR: No completion in response")
                print(json.dumps(result, indent=2))
                
        except requests.exceptions.RequestException as e:
            print(f"ERROR: Request failed: {e}")
            return False
    
    print(f"\n{'='*60}")
    print("All tests completed")
    print(f"{'='*60}")
    return True

if __name__ == "__main__":
    import sys
    success = test_sidecar()
    sys.exit(0 if success else 1)
