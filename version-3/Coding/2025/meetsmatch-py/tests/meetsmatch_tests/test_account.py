import unittest
from unittest.mock import MagicMock
from sqlalchemy.orm import Session
from src.meetsmatch.models import User, MediaFile
from src.meetsmatch.account import AccountManager
from datetime import datetime, timedelta


class TestAccountManager(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self):
        # Create a mock session factory
        self.session = MagicMock(spec=Session)
        self.session_factory = MagicMock(return_value=self.session)

        # Create the account manager with the mock session factory
        self.account_manager = AccountManager(session_factory=self.session_factory)

        # Setup common query chain mocks
        self.query_mock = MagicMock()
        self.filter_mock = MagicMock()
        self.session.query.return_value = self.query_mock
        self.query_mock.filter_by.return_value = self.filter_mock
        self.query_mock.filter.return_value = self.filter_mock

    async def test_request_deletion_success(self):
        # Create user with media files
        user = User(telegram_id=12345, is_active=True)
        media1 = MediaFile(user=user, is_deleted=False)
        media2 = MediaFile(user=user, is_deleted=False)
        user.media_files = [media1, media2]
        self.filter_mock.first.return_value = user

        success, message = await self.account_manager.request_deletion(user_id=12345)

        # Verify query used correct user_id
        self.query_mock.filter_by.assert_called_once_with(id=12345)

        # Verify success response
        self.assertTrue(success)
        self.assertEqual(
            message,
            f"Account scheduled for deletion. Will be permanently deleted in "
            f"{self.account_manager.deletion_window_days} days.",
        )

        # Verify user and media files were updated
        self.assertFalse(user.is_active)
        self.assertTrue(all(m.is_deleted for m in user.media_files))
        self.assertIsNotNone(user.updated_at)

        # Verify session was committed
        self.session.commit.assert_called_once()

    async def test_request_deletion_nonexistent_user(self):
        self.filter_mock.first.return_value = None
        success, message = await self.account_manager.request_deletion(user_id=12345)
        self.assertFalse(success)
        self.assertEqual(message, "User not found")
        self.session.commit.assert_not_called()

    async def test_request_deletion_already_inactive(self):
        user = User(telegram_id=12345, is_active=False)
        self.filter_mock.first.return_value = user
        success, message = await self.account_manager.request_deletion(user_id=12345)
        self.assertFalse(success)
        self.assertEqual(message, "Account already deactivated")
        self.session.commit.assert_not_called()

    async def test_cancel_deletion_success(self):
        # Create user with deleted media files
        user = User(telegram_id=12345, is_active=False)
        media1 = MediaFile(user=user, is_deleted=True, deleted_at=None)
        media2 = MediaFile(user=user, is_deleted=True, deleted_at=None)
        user.media_files = [media1, media2]
        user.updated_at = datetime.utcnow()
        self.filter_mock.first.return_value = user

        success, message = await self.account_manager.cancel_deletion(user_id=12345)

        # Verify query used correct user_id
        self.query_mock.filter_by.assert_called_once_with(id=12345)

        # Verify success response
        self.assertTrue(success)
        self.assertEqual(message, "Account deletion cancelled successfully")

        # Verify user and media files were restored
        self.assertTrue(user.is_active)
        self.assertTrue(
            all(not m.is_deleted and m.deleted_at is None for m in user.media_files)
        )
        self.assertIsNotNone(user.updated_at)

        # Verify session was committed
        self.session.commit.assert_called_once()

    async def test_cancel_deletion_expired(self):
        user = User(
            telegram_id=12345,
            is_active=False,
            updated_at=datetime.utcnow() - timedelta(days=31),
        )
        self.filter_mock.first.return_value = user

        success, message = await self.account_manager.cancel_deletion(user_id=12345)
        self.assertFalse(success)
        self.assertEqual(message, "Deletion cancellation period has expired")
        self.session.commit.assert_not_called()

    async def test_cancel_deletion_not_pending(self):
        user = User(telegram_id=12345, is_active=True)
        self.filter_mock.first.return_value = user

        success, message = await self.account_manager.cancel_deletion(user_id=12345)
        self.assertFalse(success)
        self.assertEqual(message, "Account is not pending deletion")
        self.session.commit.assert_not_called()

    async def test_permanent_deletion(self):
        # Create users with deletion requests
        user1 = User(
            telegram_id=12345,
            is_active=False,
            updated_at=datetime.utcnow() - timedelta(days=31),
        )
        User(telegram_id=67890, is_active=False, updated_at=datetime.utcnow())

        # Mock query for users to delete
        datetime.utcnow() - timedelta(days=30)
        self.query_mock.filter.return_value.all.return_value = [user1]

        # Mock queries for related models
        interaction_query = MagicMock()
        media_query = MagicMock()
        self.session.query.side_effect = [
            self.query_mock,  # User query
            interaction_query,  # Interaction query
            media_query,  # MediaFile query
        ]

        # Call the permanent deletion method
        await self.account_manager.permanently_delete_accounts()

        # Verify correct filter was used for users
        self.query_mock.filter.assert_called_once()

        # Verify interactions and media were deleted
        interaction_query.filter.assert_called_once()
        media_query.filter_by.assert_called_once_with(user_id=user1.id)

        # Verify user was deleted
        self.session.delete.assert_called_once_with(user1)

        # Verify session was committed
        self.session.commit.assert_called_once()
