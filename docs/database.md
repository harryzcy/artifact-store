# Database Design

The keys and what they are used for:

1. `repo#{server}#{owner}#{repo}`
    - Get all repos
1. `commit#{server}#{owner}#{repo}#{commit}`
    - Check if commit exists
1. `commit_time#{server}#{owner}#{repo}#{time}`
    - Get all commits in a repo ordered by time
1. `artifact#{commit}#{path}`
    - Get all artifacts for a commit
