from __future__ import annotations

import unittest
from unittest.mock import patch

from .main import build_parser
from .port_manifest import build_port_manifest
from .query_engine import QueryEngineConfig, QueryEnginePort
from .runtime import PortRuntime


class TurnLimitRemovalTests(unittest.TestCase):
    def test_ninth_submit_is_accepted_before_budget_stop(self) -> None:
        engine = QueryEnginePort(
            manifest=build_port_manifest(),
            config=QueryEngineConfig(max_budget_tokens=110),
        )

        results = [engine.submit_message('x') for _ in range(9)]
        final_result = engine.submit_message('x')

        self.assertEqual(len(results), 9)
        self.assertTrue(all(result.stop_reason == 'completed' for result in results))
        self.assertEqual(final_result.stop_reason, 'max_budget_reached')
        self.assertEqual(len(engine.mutable_messages), 10)

        summary = engine.render_summary()
        self.assertIn('Conversation turns stored: 10', summary)
        self.assertIn('Max budget tokens: 110', summary)
        self.assertNotIn('Max turns:', summary)

    def test_cli_rejects_removed_turn_option(self) -> None:
        parser = build_parser()
        args = parser.parse_args(['turn-loop', 'prompt', '--limit', '2', '--structured-output'])
        removed_option = '--' + 'max' + '-turns'

        self.assertEqual(args.limit, 2)
        self.assertTrue(args.structured_output)
        with self.assertRaises(SystemExit) as raised:
            parser.parse_args(['turn-loop', 'prompt', removed_option])
        self.assertEqual(raised.exception.code, 2)

    def test_turn_loop_stops_at_budget_not_default_count(self) -> None:
        engine = QueryEnginePort(
            manifest=build_port_manifest(),
            config=QueryEngineConfig(max_budget_tokens=100),
        )

        with patch.object(QueryEnginePort, 'from_workspace', return_value=engine):
            results = PortRuntime().run_turn_loop('x', limit=0)

        self.assertGreater(len(results), 3)
        self.assertTrue(all(result.stop_reason == 'completed' for result in results[:-1]))
        self.assertEqual(results[-1].stop_reason, 'max_budget_reached')


if __name__ == '__main__':
    unittest.main()
