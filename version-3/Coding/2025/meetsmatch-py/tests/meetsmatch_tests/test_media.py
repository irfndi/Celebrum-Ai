from datetime import datetime, timedelta
from unittest.mock import patch
from moto import mock_s3
import boto3
from meetsmatch.models import MediaFile, User
from meetsmatch.media import MediaHandler
from meetsmatch_tests.test_base import TestBase
import pytest

class TestMediaHandler(TestBase):
    """Test media handling methods."""

    async def asyncSetUp(self):
        """Setup test dependencies."""
        await super().asyncSetUp()

        # Configure S3 mock with moto
        self.mock_s3 = mock_s3()
        self.mock_s3.start()
        
        # Initialize S3 client and create test bucket
        self.s3_client = boto3.client("s3", region_name="us-east-1")
        self.s3_client.create_bucket(Bucket="test-bucket")
        
        # Create MediaHandler instance with proper mocks
        self.media_handler = MediaHandler(
            session_factory=self.Session,
            bucket_name="test-bucket",
            max_files=5,
            # 5MB
            max_size_bytes=5_242_880,
        )

        # Create test user
        self.user = User(
            telegram_id=9876543210,
            username="test_user",
            name="Test User",
            age=25,
            gender="male",
            location="Test City",
            bio="Test bio",
            interests='["coding"]'
        )
        async with self.Session() as session:
            session.add(self.user)
            await session.commit()

        self.addAsyncCleanup(self.mock_s3.stop)

    # --- Fixed Test Cases with Proper Async Handling ---
    @pytest.mark.asyncio
    async def test_save_media_success(self):
        """Test successful media file save"""
        with patch.object(self.media_handler, '_generate_s3_key', return_value="test-key"):
            media_file = await self.media_handler.save_media(
                self.user.id, 
                b"test-content",
                "image/jpeg"
            )
            
            assert media_file.user_id == self.user.id
            assert media_file.file_type == "image/jpeg"
            assert media_file.size_bytes == 11

    @pytest.mark.asyncio
    async def test_media_minimum_required(self):
        """Test minimum 1 media requirement"""
        with self.assertRaises(ValueError):
            await self.media_handler.validate_media_requirements(self.user.id)

    @pytest.mark.asyncio
    async def test_media_type_validation(self):
        """Test only images/videos allowed"""
        with self.assertRaises(ValueError):
            await self.media_handler.save_media(
                self.user.id,
                b"content",
                "application/pdf"
            )

    @pytest.mark.asyncio
    async def test_s3_deletion_workflow(self):
        """Test 180-day deletion process"""
        # Create test media
        media = MediaFile(
            user_id=self.user.id,
            file_type="image/jpeg",
            s3_key="old-file",
            created_at=datetime.now() - timedelta(days=181)
        )
        async with self.Session() as session:
            session.add(media)
            await session.commit()

        # Upload dummy file to S3
        self.s3_client.put_object(
            Bucket="test-bucket",
            Key="old-file",
            Body=b"test"
        )

        # Run cleanup
        deleted_count = await self.media_handler.delete_old_media()
        assert deleted_count == 1

        # Verify S3 deletion
        with self.assertRaises(self.s3_client.exceptions.NoSuchKey):
            self.s3_client.head_object(
                Bucket="test-bucket",
                Key="old-file"
            )

    # ... (other test cases with similar fixes)
