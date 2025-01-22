from datetime import datetime, timedelta
from typing import List, Tuple
from sqlalchemy import func
from .models import Session, User, Interaction


class ReportManager:
    def __init__(self):
        self.report_threshold = 3  # Number of reports before auto-ban
        self.report_window_days = 30  # Time window for counting reports
        self.report_reasons = [
            "inappropriate_content",
            "harassment",
            "spam",
            "fake_profile",
            "underage",
            "other",
        ]

    async def report_user(
        self, reporter_id: int, reported_id: int, reason: str
    ) -> Tuple[bool, str]:
        """
        Report a user for inappropriate behavior.

        Args:
            reporter_id (int): ID of the user making the report
            reported_id (int): ID of the user being reported
            reason (str): Reason for the report

        Returns:
            Tuple[bool, str]: (Success status, Message)
        """
        if reason not in self.report_reasons:
            return False, "Invalid report reason"

        session = Session()
        try:
            # Check if reporter exists
            reporter = session.query(User).filter_by(id=reporter_id).first()
            if not reporter:
                return False, "Reporter not found"

            # Check if reported user exists
            reported = session.query(User).filter_by(id=reported_id).first()
            if not reported:
                return False, "Reported user not found"

            # Check if already reported by this user
            existing_report = (
                session.query(Interaction)
                .filter(
                    Interaction.user_id == reporter_id,
                    Interaction.target_user_id == reported_id,
                    Interaction.interaction_type == "report",
                    Interaction.created_at
                    > datetime.utcnow() - timedelta(days=self.report_window_days),
                )
                .first()
            )

            if existing_report:
                return False, "You have already reported this user"

            # Create report
            report = Interaction(
                user_id=reporter_id,
                target_user_id=reported_id,
                interaction_type="report",
                report_reason=reason,
                created_at=datetime.utcnow(),
            )
            session.add(report)

            # Check if user should be banned
            report_count = (
                session.query(func.count(Interaction.id))
                .filter(
                    Interaction.target_user_id == reported_id,
                    Interaction.interaction_type == "report",
                    Interaction.created_at
                    > datetime.utcnow() - timedelta(days=self.report_window_days),
                )
                .scalar()
            )

            if report_count + 1 >= self.report_threshold:
                reported.is_active = False
                message = "User has been banned due to multiple reports"
            else:
                message = "Report submitted successfully"

            session.commit()
            return True, message

        except Exception as e:
            session.rollback()
            return False, f"Error submitting report: {str(e)}"
        finally:
            session.close()

    async def get_user_reports(self, user_id: int) -> List[Interaction]:
        """Get all reports for a specific user."""
        session = Session()
        try:
            return (
                session.query(Interaction)
                .filter(
                    Interaction.target_user_id == user_id,
                    Interaction.interaction_type == "report",
                )
                .all()
            )
        finally:
            session.close()

    async def get_reports_by_reason(self, days: int = 30) -> dict:
        """Get report statistics grouped by reason."""
        session = Session()
        try:
            reports = (
                session.query(Interaction.report_reason, func.count(Interaction.id))
                .filter(
                    Interaction.interaction_type == "report",
                    Interaction.created_at > datetime.utcnow() - timedelta(days=days),
                )
                .group_by(Interaction.report_reason)
                .all()
            )

            return dict(reports)
        finally:
            session.close()

    async def get_banned_users(self) -> List[User]:
        """Get list of banned users."""
        session = Session()
        try:
            return session.query(User).filter(User.is_active is False).all()
        finally:
            session.close()


# Global instance
report_manager = ReportManager()
