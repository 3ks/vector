package metadata

installation: platforms: {
	docker: {
		title:                     "Docker"
		description:               """
			[Docker](\(urls.docker)) is an open platform for developing, shipping, and running
			applications and services. Docker enables you to separate your services from
			your infrastructure so you can ship quickly. With Docker, you can manage your
			infrastructure in the same ways you manage your services. By taking advantage
			of Docker's methodologies for shipping, testing, and deploying code quickly,
			you can significantly reduce the delay between writing code and running it in
			production.
			"""
		minimum_supported_version: null
	}

	kubernetes: {
		title:                     "Kubernetes"
		description:               """
			[Kubernetes](\(urls.kubernetes)), also known as k8s, is an
			open-source container-orchestration system for automating
			application deployment, scaling, and management.
			"""
		minimum_supported_version: "1.14"
	}
}
