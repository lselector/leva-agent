"""Job management endpoints."""

from fastapi import APIRouter, HTTPException
from pydantic import BaseModel
from typing import Optional
from .job_runner import runner

router = APIRouter(prefix="/jobs")


# --------------------------------------------------------------
class JobStartRequest(BaseModel):
    """Request to start a job."""
    type: str
    params: Optional[dict] = {}


# --------------------------------------------------------------
@router.post("/start")
async def start_job(req: JobStartRequest):
    """Start a new background job."""
    job = runner.start_job(
        req.type, req.params or {}
    )
    return {
        "job_id": job.job_id,
        "status": job.status,
    }


# --------------------------------------------------------------
@router.get("/list")
async def list_jobs():
    """List all jobs."""
    return runner.list_jobs()


# --------------------------------------------------------------
@router.get("/status/{job_id}")
async def job_status(job_id: str):
    """Get status of a specific job."""
    job = runner.get_job(job_id)
    if job is None:
        raise HTTPException(
            status_code=404,
            detail="Job not found",
        )
    return job.to_dict()
