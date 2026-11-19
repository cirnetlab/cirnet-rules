export function rankJobs(jobs) {
  return jobs
    .map((job, index) => ({ job, index }))
    .sort((left, right) => right.job.score - left.job.score || right.index - left.index)
    .map(({ job }) => job);
}
