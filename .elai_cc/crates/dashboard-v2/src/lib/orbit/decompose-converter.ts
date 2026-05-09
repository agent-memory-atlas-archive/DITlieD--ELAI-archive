// decompose-converter.ts
// Pure conversion from the live decomposition tree (plan store) into the
// OrbitFixture shape consumed by OrbitCanvas. Replaces the static mock
// fixtures used during PART8 development.

import type { DecompNode, DecompLevel, FsmState } from '../api/types';
import type {
  OrbitFixture,
  ConceptStub,
  PhaseStub,
  StepStub,
  BabyStepStub,
} from './types';

function idleFixture(fsmState: FsmState): OrbitFixture {
  return {
    task_id: 'idle',
    idea: 'No active task',
    fsm: fsmState,
    concepts: [],
  };
}

function toBabyStepStub(node: DecompNode): BabyStepStub {
  return {
    id: node.id,
    label: node.label,
    task: node.label,
  };
}

function toStepStub(node: DecompNode): StepStub {
  const children = node.children ?? [];
  const babySteps: BabyStepStub[] = [];
  for (const c of children) {
    if (c.level === 'baby-step') {
      babySteps.push(toBabyStepStub(c));
    } else {
      console.warn('[decompose-converter] expected baby-step child of step', { stepId: node.id, gotLevel: c.level });
    }
  }
  return { id: node.id, label: node.label, baby_steps: babySteps };
}

function toPhaseStub(node: DecompNode): PhaseStub {
  const children = node.children ?? [];
  const steps: StepStub[] = [];
  for (const c of children) {
    if (c.level === 'step') {
      steps.push(toStepStub(c));
    } else {
      console.warn('[decompose-converter] expected step child of phase', { phaseId: node.id, gotLevel: c.level });
    }
  }
  return { id: node.id, label: node.label, steps };
}

function toConceptStub(node: DecompNode): ConceptStub {
  const children = node.children ?? [];
  const phases: PhaseStub[] = [];
  for (const c of children) {
    if (c.level === 'phase') {
      phases.push(toPhaseStub(c));
    } else {
      console.warn('[decompose-converter] expected phase child of concept', { conceptId: node.id, gotLevel: c.level });
    }
  }
  return { id: node.id, name: node.label, phases };
}

// Walk down the tree along the active branch (status='active' wins; else
// first child) until reaching a node at the requested level. Returns
// undefined if the tree doesn't reach that depth.
function findActiveIdAt(root: DecompNode, target: DecompLevel): string | undefined {
  if (root.level === target) return root.id;
  const children = root.children ?? [];
  if (children.length === 0) return undefined;
  const next = children.find((c) => c.status === 'active') ?? children[0];
  if (!next) return undefined;
  return findActiveIdAt(next, target);
}

export function decompNodeToOrbitFixture(tree: DecompNode, fsmState: FsmState): OrbitFixture {
  // Idle sentinel — empty tree placeholder from plan.svelte.ts.
  if (tree.id === 'empty') {
    return idleFixture(fsmState);
  }
  // Defensive: the convention is that the root is a concept-level node.
  // If something else lands here, log and degrade to idle rather than crash.
  if (tree.level !== 'concept') {
    console.warn('[decompose-converter] tree root is not concept-level', { rootLevel: tree.level });
    return idleFixture(fsmState);
  }

  const concept = toConceptStub(tree);
  const concepts: ConceptStub[] = [concept];

  return {
    task_id: tree.id,
    idea: tree.label,
    fsm: fsmState,
    active_concept_id: concept.id,
    active_phase_id: findActiveIdAt(tree, 'phase'),
    active_step_id: findActiveIdAt(tree, 'step'),
    active_baby_step_id: findActiveIdAt(tree, 'baby-step'),
    concepts,
  };
}
