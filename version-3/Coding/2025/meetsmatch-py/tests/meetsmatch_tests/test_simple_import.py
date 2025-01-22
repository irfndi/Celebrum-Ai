from sqlalchemy.ext.asyncio import create_async_engine, AsyncSession
from sqlalchemy.orm import sessionmaker
from meetsmatch.matching import Matcher


async def test_simple_import():
    """Test that the Matcher class can be imported and instantiated."""
    # Create an async engine and session for testing
    engine = create_async_engine("sqlite+aiosqlite:///:memory:")
    async_session = sessionmaker(engine, expire_on_commit=False, class_=AsyncSession)

    # Instantiate Matcher with a session
    async with async_session() as session:
        matcher = Matcher(session)
        assert matcher is not None

    # Dispose the engine
    await engine.dispose()
