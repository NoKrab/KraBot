pipeline {
    agent {
        label 'RHEL'
    }

    stages {
        stage('Build') {
            steps {
                sh "cargo build"
            }
        }
    }
}