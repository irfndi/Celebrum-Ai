from datetime import datetime, timedelta
from typing import Tuple
from .models import Session, User, MediaFile, Interaction
from .media import media_handler


class AccountManager:
    def __init__(self, session_factory=None):
        self.deletion_window_days = 30  # Days to keep data before permanent deletion
        self.session_factory = session_factory or Session

    async def request_deletion(self, user_id: int) -> Tuple[bool, str]:
        """
        Request account deletion. Account will be deactivated immediately and
        permanently deleted after deletion_window_days.

        Args:
            user_id (int): ID of the user requesting deletion

        Returns:
            Tuple[bool, str]: (Success status, Message)
        """
        session = self.session_factory()
        try:
            user = session.query(User).filter_by(id=user_id).first()
            if not user:
                return False, "User not found"

            if not user.is_active:
                return False, "Account already deactivated"

            # Deactivate account
            user.is_active = False
            user.updated_at = datetime.utcnow()

            # Mark all media for deletion
            for media in user.media_files:
                if not media.is_deleted:
                    media.is_deleted = True
                    media.deleted_at = datetime.utcnow()

            session.commit()
            return True, (
                f"Account scheduled for deletion. "
                f"Will be permanently deleted in {self.deletion_window_days} days."
            )

        except Exception as e:
            session.rollback()
            return False, f"Error requesting deletion: {str(e)}"
        finally:
            session.close()

    async def cancel_deletion(self, user_id: int) -> Tuple[bool, str]:
        """Cancel a pending account deletion request."""
        session = self.session_factory()
        try:
            user = session.query(User).filter_by(id=user_id).first()
            if not user:
                return False, "User not found"

            if user.is_active:
                return False, "Account is not pending deletion"

            # Check if within cancellation window
            if user.updated_at < datetime.utcnow() - timedelta(
                days=self.deletion_window_days
            ):
                return False, "Deletion cancellation period has expired"

            # Reactivate account
            user.is_active = True
            user.updated_at = datetime.utcnow()

            # Restore media files
            for media in user.media_files:
                if media.is_deleted and not media.deleted_at:
                    media.is_deleted = False
                    media.deleted_at = None

            session.commit()
            return True, "Account deletion cancelled successfully"

        except Exception as e:
            session.rollback()
            return False, f"Error cancelling deletion: {str(e)}"
        finally:
            session.close()

    async def permanently_delete_accounts(self) -> None:
        """Permanently delete accounts that have been marked for deletion and
        passed the deletion window."""
        session = self.session_factory()
        try:
            deletion_date = datetime.utcnow() - timedelta(
                days=self.deletion_window_days
            )

            # Find accounts to delete
            users_to_delete = (
                session.query(User)
                .filter(User.is_active is False, User.updated_at <= deletion_date)
                .all()
            )

            for user in users_to_delete:
                # Delete media files from S3
                for media in user.media_files:
                    try:
                        await media_handler.delete_file(media.s3_key)
                    except Exception as e:
                        print(f"Error deleting media {media.s3_key}: {str(e)}")

                # Delete interactions
                session.query(Interaction).filter(
                    (Interaction.user_id == user.id)
                    | (Interaction.target_user_id == user.id)
                ).delete(synchronize_session=False)

                # Delete media records
                session.query(MediaFile).filter_by(user_id=user.id).delete(
                    synchronize_session=False
                )

                # Delete user
                session.delete(user)

            session.commit()

        except Exception as e:
            session.rollback()
            print(f"Error in permanently_delete_accounts: {str(e)}")
        finally:
            session.close()


# Global instance
account_manager = AccountManager()
