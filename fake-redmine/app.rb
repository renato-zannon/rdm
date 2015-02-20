require 'sinatra'
require 'json'

ISSUE_STATUSES = [
  { id: 1, name: "Solved"      },
  { id: 2, name: "Rejected"    },
  { id: 3, name: "In Progress" },
  { id: 4, name: "Interrupted" },
]

after do
  logger.info params.inspect
end

get '/issue_statuses.json' do
  { issue_statuses: ISSUE_STATUSES }.to_json
end

put '/issues/:id.json' do
  [200, {}, []]
end
