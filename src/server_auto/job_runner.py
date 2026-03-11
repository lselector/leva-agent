"""Simple in-memory job runner."""

import uuid
import time
import threading
from typing import Dict, Any, Optional


# --------------------------------------------------------------
class Job:
    """Represents a background job."""

    def __init__(self, job_type, params):
        """Create a new job."""
        self.job_id = str(uuid.uuid4())[:8]
        self.job_type = job_type
        self.params = params
        self.status = "pending"
        self.result = None
        self.error = None
        self.created_at = time.time()
        self.completed_at = None

    def to_dict(self):
        """Convert job to dictionary."""
        return {
            "job_id": self.job_id,
            "type": self.job_type,
            "status": self.status,
            "result": self.result,
            "error": self.error,
            "created_at": self.created_at,
            "completed_at": self.completed_at,
        }


# --------------------------------------------------------------
class JobRunner:
    """Manages background jobs."""

    def __init__(self):
        """Initialize job runner."""
        self.jobs: Dict[str, Job] = {}

    def start_job(
        self, job_type: str, params: dict
    ) -> Job:
        """Start a new background job."""
        job = Job(job_type, params)
        self.jobs[job.job_id] = job
        job.status = "running"
        thread = threading.Thread(
            target=self._run_job,
            args=(job,),
            daemon=True,
        )
        thread.start()
        return job

    def _run_job(self, job: Job):
        """Execute a job in background."""
        try:
            if job.job_type == "test":
                time.sleep(1)
                job.result = "test completed"
            else:
                job.result = (
                    f"unknown type: "
                    f"{job.job_type}"
                )
            job.status = "completed"
        except Exception as e:
            job.status = "failed"
            job.error = str(e)
        finally:
            job.completed_at = time.time()

    def get_job(self, job_id: str):
        """Get a job by ID."""
        return self.jobs.get(job_id)

    def list_jobs(self):
        """List all jobs."""
        return [
            j.to_dict()
            for j in self.jobs.values()
        ]


runner = JobRunner()
