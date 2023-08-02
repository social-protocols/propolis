sqlite-vector-version = 1fdc48a773808392fe0776b66c4ac0aa2d18a068

.DELETE_ON_ERROR:

vector0.so: sqlite-vector 
	cd sqlite-vector && cmake .
	cd sqlite-vector && make
	cp sqlite-vector/vector0.so .
	
sqlite-vector:
	mkdir -p sqlite-vector
	# shallow clone of specific commit
	# https://stackoverflow.com/questions/31278902/how-to-shallow-clone-a-specific-commit-with-depth-1/43136160#43136160
	cd sqlite-vector && git -c init.defaultBranch=main init
	cd sqlite-vector && git remote add origin https://github.com/asg017/sqlite-vector.git
	cd sqlite-vector && git fetch --depth 1 origin $(sqlite-vector-version)
	cd sqlite-vector && git -c advice.detachedHead=false checkout FETCH_HEAD
	# get all submodules, shallow
	cd sqlite-vector && git submodule update --init --recursive --depth 1

clean:
	rm -rf sqlite-vector
	rm -f vector0.so
