docker rmi $(docker images -qa -f 'dangling=true')
