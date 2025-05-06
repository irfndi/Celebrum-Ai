from setuptools import setup

setup(
    name="meetsmatch",
    version="0.1",
    package_dir={"": "src"},  # This means the packages are rooted in src/
    packages=["meetsmatch"],  # This means look for the meetsmatch package in src/
    install_requires=[
        "python-telegram-bot==20.3",
        "sqlalchemy==2.0.23",
        "psycopg2-binary==2.9.9",
        "boto3==1.28.62",
        "uv==0.1.0",
        "ruff==0.0.292",
        "geopy==2.4.1",
        "python-dotenv==1.0.0",
        "coverage==7.6.10",
        "babel==2.14.0",
        "pytest-asyncio==0.23.5",
        "python-Levenshtein==0.23.0",
        "thefuzz==0.20.0",
        "pillow==10.1.0",
        "python-magic==0.4.27",
        "aiohttp==3.11.0b0",
    ],
)
