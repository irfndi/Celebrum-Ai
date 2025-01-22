import unittest
from meetsmatch.models import Session, User, Interaction
from meetsmatch.reports import ReportManager


class TestReportManager(unittest.TestCase):
    def setUp(self):
        self.session = Session()
        self.report_manager = ReportManager()

        # Create test users with unique telegram_ids
        self.reporter = User(telegram_id=2001, username="reporter")
        self.reported = User(telegram_id=2002, username="reported")
        self.session.add_all([self.reporter, self.reported])
        self.session.commit()

    def tearDown(self):
        self.session.query(Interaction).delete()
        self.session.query(User).delete()
        self.session.commit()
        self.session.close()

    async def test_report_user_success(self):
        """Test successful user report."""
        result, message = await self.report_manager.report_user(
            self.reporter.id, self.reported.id, "inappropriate_content"
        )
        self.assertTrue(result)
        self.assertEqual(message, "Report submitted successfully")

    async def test_report_user_invalid_reason(self):
        """Test report with invalid reason."""
        result, message = await self.report_manager.report_user(
            self.reporter.id, self.reported.id, "invalid_reason"
        )
        self.assertFalse(result)
        self.assertEqual(message, "Invalid report reason")

    async def test_report_user_duplicate(self):
        """Test duplicate report from same user."""
        # First report
        await self.report_manager.report_user(
            self.reporter.id, self.reported.id, "inappropriate_content"
        )

        # Second report
        result, message = await self.report_manager.report_user(
            self.reporter.id, self.reported.id, "inappropriate_content"
        )
        self.assertFalse(result)
        self.assertEqual(message, "You have already reported this user")

    async def test_auto_ban(self):
        """Test auto-ban after threshold reports."""
        # Create additional reporters
        reporters = []
        for i in range(3):
            reporter = User(telegram_id=100 + i, username=f"reporter{i}")
            self.session.add(reporter)
            reporters.append(reporter)
        self.session.commit()

        # Submit reports from different users
        for reporter in reporters:
            result, message = await self.report_manager.report_user(
                reporter.id, self.reported.id, "inappropriate_content"
            )
            self.assertTrue(result)

        # Check if user is banned
        reported_user = self.session.query(User).filter_by(id=self.reported.id).first()
        self.assertFalse(reported_user.is_active)

    async def test_report_user_flags_profile(self):
        """Test that reported profiles are flagged."""
        await self.report_manager.report_user(
            self.reporter.id, self.reported.id, "inappropriate_content"
        )
        reported_user = self.session.query(User).filter_by(id=self.reported.id).first()
        self.assertTrue(reported_user.is_reported)

    async def test_get_reports_by_reason(self):
        """Test report statistics."""
        # Submit various reports
        await self.report_manager.report_user(
            self.reporter.id, self.reported.id, "inappropriate_content"
        )

        stats = await self.report_manager.get_reports_by_reason()
        self.assertIn("inappropriate_content", stats)
        self.assertEqual(stats["inappropriate_content"], 1)


if __name__ == "__main__":
    unittest.main()
