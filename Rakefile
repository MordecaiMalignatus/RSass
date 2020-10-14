task :default => :test

task :test do
  qsh 'cargo test'
end

task :build do
  qsh 'cargo build --release'
end

task :install => :build do
  mv 'target/release/rsass', "#{ENV['HOME']}/.cargo/bin/rsass", force: true
  chmod '+x', "#{ENV['HOME']}/.cargo/bin/rsass"
end

def qsh(*cmd)
  sh(*cmd, :verbose => false)
end
