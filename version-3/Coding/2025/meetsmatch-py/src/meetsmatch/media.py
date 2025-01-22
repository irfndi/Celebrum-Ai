import boto3
from datetime import datetime, timedelta, timezone
from typing import Optional, List, Dict, Any
from .models import MediaFile, Session
from .config import Config
from sqlalchemy import select, func, and_, not_
import uuid


class MediaHandler:
    def __init__(self, session_factory, s3_client=None):
        self.session_factory = session_factory
        if s3_client:
            self.s3 = s3_client
        else:
            self.s3 = boto3.client(
                "s3",
                aws_access_key_id=Config.S3_ACCESS_KEY,
                aws_secret_access_key=Config.S3_SECRET_KEY,
                region_name=Config.S3_REGION,
                endpoint_url=Config.S3_ENDPOINT,
            )
        self.bucket_name = Config.S3_BUCKET
        self.max_size_bytes = Config.MAX_MEDIA_SIZE
        self.max_files = Config.MAX_MEDIA_FILES

    async def get_user_media(self, user_id: int) -> List[MediaFile]:
        """Get all active media files for a user."""
        async with self.session_factory() as session:
            async with session.begin():
                result = await session.execute(
                    select(MediaFile).where(
                        and_(MediaFile.user_id == user_id, not_(MediaFile.is_deleted))
                    )
                )
                return list(result.scalars().all())

    async def get_user_media_files(
        self, user_id: int, file_type: Optional[str] = None
    ) -> List[Dict[str, Any]]:
        """Get all media files for a user, optionally filtered by type."""
        async with self.session_factory() as session:
            query = session.query(MediaFile).filter(
                MediaFile.user_id == user_id, not_(MediaFile.is_deleted)
            )

            if file_type:
                query = query.filter(MediaFile.file_type == file_type)

            media_files = await query.all()
            return [
                {
                    "id": media.id,
                    "file_type": media.file_type,
                    "file_id": media.file_id,
                    "size_bytes": media.size_bytes,
                    "created_at": media.created_at,
                }
                for media in media_files
            ]

    async def save_media(
        self, user_id: int, file_data: bytes, file_type: str
    ) -> Optional[MediaFile]:
        """
        Save a media file for a user.

        Args:
            user_id (int): The user ID
            file_data (bytes): The file data
            file_type (str): The type of file (image/video)

        Returns:
            Optional[MediaFile]: The saved media file or None if error
        """
        try:
            # Check file size
            if len(file_data) > self.max_size_bytes:
                raise ValueError("File too large")

            # Check number of files
            async with self.session_factory() as session:
                async with session.begin():
                    active_files = await session.execute(
                        select(func.count(MediaFile.id)).where(
                            and_(
                                MediaFile.user_id == user_id, not_(MediaFile.is_deleted)
                            )
                        )
                    )
                    count = active_files.scalar()
                    if count >= self.max_files:
                        raise ValueError("Maximum number of files reached")

            # Generate unique file ID and S3 key
            file_id = str(uuid.uuid4())
            s3_key = f"{user_id}/{file_id}"

            # Upload to S3
            try:
                await self.s3.put_object(
                    Bucket=self.bucket_name,
                    Key=s3_key,
                    Body=file_data,
                    ContentType=f"{file_type}/jpeg",
                )
            except Exception as e:
                raise e

            # Save file metadata
            media_file = await self.save_media_file(
                user_id, file_type, file_id, s3_key, len(file_data)
            )
            return media_file

        except Exception as e:
            raise e

    async def save_media_file(
        self, user_id: int, file_type: str, file_id: str, s3_key: str, size_bytes: int
    ) -> MediaFile:
        """Save media file metadata to database."""
        media = MediaFile(
            user_id=user_id,
            file_type=file_type,
            file_id=file_id,
            s3_key=s3_key,
            size_bytes=size_bytes,
            created_at=datetime.now(timezone.utc),
            is_deleted=False,
        )
        async with self.session_factory() as session:
            async with session.begin():
                session.add(media)
                await session.commit()
                return media

    async def get_download_url(self, media_file: MediaFile) -> str:
        """
        Get a pre-signed download URL for a media file.

        Args:
            media_file (MediaFile): The media file

        Returns:
            str: Pre-signed URL
        """
        return await self.s3.generate_presigned_url(
            "get_object",
            Params={
                "Bucket": self.bucket_name,
                "Key": media_file.s3_key,
            },
            ExpiresIn=3600,  # URL valid for 1 hour
        )

    async def delete_old_media(self, days_old: int = 180) -> None:
        """Delete media files older than specified days."""
        cutoff_date = datetime.utcnow() - timedelta(days=days_old)

        # Find old media files
        async with self.session_factory() as session:
            async with session.begin():
                result = await session.execute(
                    select(MediaFile).where(MediaFile.created_at < cutoff_date)
                )
                old_files = result.scalars().all()

                # Delete from S3 and update database
                for media in old_files:
                    try:
                        await self.s3.delete_object(
                            Bucket=self.bucket_name, Key=media.s3_key
                        )
                        media.is_deleted = True
                    except Exception as e:
                        print(f"Error deleting {media.s3_key}: {e}")

                await session.commit()
                return len(old_files)

    async def delete_old_media_after_change(self) -> int:
        """Delete old media files after a change."""
        return await self.delete_old_media()


# Global instance
media_handler = MediaHandler(session_factory=Session)
